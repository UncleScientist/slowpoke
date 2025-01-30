use std::{
    cell::RefCell,
    collections::HashMap,
    io::Stdout,
    time::{Duration, Instant},
};

use clamp_to::{Clamp, ClampTo};
use crossterm::{
    event::{KeyboardEnhancementFlags, MouseEventKind},
    execute,
};
use either::Either;
use lyon_tessellation::{
    geom::{euclid::default::Transform2D, point, Angle, Point},
    geometry_builder::simple_builder,
    path::Path,
    FillOptions, FillTessellator, VertexBuffers,
};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Position, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::Line as TextLine,
    widgets::{
        canvas::{Canvas, Circle, Context, Line, Painter},
        Block, Borders, Paragraph,
    },
    Frame, Terminal,
};

use crate::{
    color_names::TurtleColor,
    generate::{CirclePos, DrawCommand},
    polygon::{PolygonPath, TurtleShape},
    turtle::{
        task::TurtleTask,
        types::{PopupID, TurtleID},
        TurtleFlags,
    },
};

use super::{events::TurtleEvent, popup::PopupData, StampCount, TurtleGui};

pub(crate) struct RatatuiFramework {
    tt: TurtleTask,
    tui: RatatuiInternal,
}

#[derive(Default)]
struct IndividualTurtle {
    cmds: Vec<DrawCommand>,
    drawing: Vec<RatatuiDrawCmd>,
    has_new_cmd: bool,
    turtle_shape: TurtleShape,
    hide_turtle: bool,
}

struct CircleDrawData {
    line_list: Vec<([f32; 2], [f32; 2])>,
    position: [f32; 2],
    angle: f32,
}

impl IndividualTurtle {
    fn draw(&self, ctx: &mut Context) -> Vec<(f32, f32, &String, &Color)> {
        let mut text_draw_cmds = Vec::new();
        for cmd in &self.drawing {
            match cmd {
                RatatuiDrawCmd::Line(l) => ctx.draw(l),
                RatatuiDrawCmd::Circle(c) => ctx.draw(c),
                RatatuiDrawCmd::Text { x, y, text, color } => {
                    let painter: Painter = ctx.into();
                    if let Some((x, y)) = painter.get_point(x.clamp_to(), y.clamp_to()) {
                        // x/2 & y/4 because that's the size of the Marker::Braille dots
                        let x = x.clamp_to_f32() / 2.;
                        let y = y.clamp_to_f32() / 4.;
                        text_draw_cmds.push((x, y, text, color));
                    }
                }
            }
        }
        text_draw_cmds
    }

    fn convert(&mut self, pct: f32) {
        let mut _penwidth = 1f32;
        let pct = f64::from(pct);

        let mut pencolor = TurtleColor::default();
        let mut fillcolor = TurtleColor::default();
        let mut trot = 0f32;
        let mut tpos = [0f64, 0f64];
        let mut iter = self.cmds.iter().peekable();

        self.drawing.clear();

        while let Some(cmd) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            match cmd {
                DrawCommand::Line(l) => {
                    let (begin_x, begin_y): (f64, f64) =
                        (l.begin.x.clamp_to(), l.begin.y.clamp_to());
                    let (end_x, end_y) = if last_element {
                        let end_x =
                            l.begin.x.clamp_to_f64() + (l.end.x - l.begin.x).clamp_to_f64() * pct;
                        let end_y =
                            l.begin.y.clamp_to_f64() + (l.end.y - l.begin.y).clamp_to_f64() * pct;
                        tpos = [end_x, end_y];
                        (tpos[0], tpos[1])
                    } else {
                        tpos = [f64::from(l.end.x), f64::from(l.end.y)];
                        (tpos[0], tpos[1])
                    };
                    if l.pen_down {
                        self.drawing.push(RatatuiDrawCmd::line(
                            (begin_x, begin_y),
                            (end_x, end_y),
                            (&pencolor).into(),
                        ));
                    }
                }
                DrawCommand::Filler | DrawCommand::Filled(_) => {}
                DrawCommand::SetPenColor(pc) => pencolor = *pc,
                DrawCommand::SetPenWidth(pw) => _penwidth = *pw,
                DrawCommand::SetFillColor(fc) => fillcolor = *fc,
                DrawCommand::SetPosition(pos) => {
                    tpos = [pos.x.clamp_to(), pos.y.clamp_to()];
                }
                DrawCommand::DrawPolygon(p) => {
                    let path = p.get_path();
                    for triangle in path.as_slice().windows(3) {
                        let lines = get_fill_lines(triangle);
                        for line in lines {
                            self.drawing.push(RatatuiDrawCmd::line(
                                (line.0.x.into(), line.0.y.into()),
                                (line.1.x.into(), line.1.y.into()),
                                (&fillcolor).into(),
                            ));
                        }
                    }
                }
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * pct.clamp_to_f32()
                    } else {
                        *end
                    };
                    trot = rotation;
                }
                DrawCommand::Dot(center, radius, color) => {
                    self.drawing.push(RatatuiDrawCmd::circle(
                        (f64::from(center.x), f64::from(center.y)),
                        f64::from(*radius),
                        color.into(),
                    ));
                }
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let angle = Angle::degrees(*angle);
                    let pos = [pos.x, pos.y];
                    let transform = Transform2D::rotation(angle).then_translate(pos.into());
                    let path = polygon.get_path();

                    for triangle in path.as_slice().windows(3) {
                        let lines = get_fill_lines(triangle);
                        for pair in lines {
                            let p1 = pair.0;
                            let p2 = pair.1;
                            let start = transform.transform_point(p1);
                            let end = transform.transform_point(p2);
                            self.drawing.push(RatatuiDrawCmd::line(
                                (start.x as f64, start.y as f64),
                                (end.x as f64, end.y as f64),
                                (&pencolor).into(),
                            ));
                        }
                    }
                }
                DrawCommand::Circle(points) => {
                    let CircleDrawData {
                        line_list,
                        position,
                        angle,
                    } = Self::circle_path(last_element, pct.clamp_to(), points);
                    tpos = [position[0] as f64, position[1] as f64];
                    trot = angle;
                    for line in line_list {
                        self.drawing.push(RatatuiDrawCmd::line(
                            (line.0[0] as f64, line.0[1] as f64),
                            (line.1[0] as f64, line.1[1] as f64),
                            (&pencolor).into(),
                        ));
                    }
                }
                DrawCommand::Text(pos, text) => {
                    self.drawing
                        .push(RatatuiDrawCmd::text(pos, text, pencolor.into()));
                }
                DrawCommand::StampTurtle
                | DrawCommand::Clear
                | DrawCommand::Reset
                | DrawCommand::BeginFill
                | DrawCommand::EndFill
                | DrawCommand::BeginPoly
                | DrawCommand::EndPoly => panic!("invalid draw command in gui"),
            }
        }

        if !self.hide_turtle {
            let angle = Angle::degrees(trot);
            let tpos = [tpos[0] as f32, tpos[1] as f32];
            let transform = Transform2D::rotation(angle).then_translate(tpos.into());
            for poly in &self.turtle_shape.poly {
                for pair in poly.polygon.path.as_slice().windows(2) {
                    let p1 = pair[0];
                    let p2 = pair[1];
                    let start = transform.transform_point(p1.into());
                    let end = transform.transform_point(p2.into());

                    let pencolor = pencolor.color_or(&poly.outline);
                    let _fillcolor = fillcolor.color_or(&poly.fill);

                    self.drawing.push(RatatuiDrawCmd::line(
                        (start.x as f64, start.y as f64),
                        (end.x as f64, end.y as f64),
                        (&pencolor).into(),
                    ));
                }
            }
        }
    }

    // returns path, final point, and final angle
    fn circle_path(last_element: bool, pct: f32, points: &[CirclePos]) -> CircleDrawData {
        let mut line_list = Vec::new();

        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut position = [0., 0.];
        let mut angle = 0.;
        let (_, mut start) = points[0].get_data();

        let mut iter = points.windows(2).take(total + 1).peekable();
        while let Some(p) = iter.next() {
            let (end_angle, end) = p[1].get_data();
            let last_segment = iter.peek().is_none();
            position = end;
            if last_element && last_segment {
                let (_, begin) = p[0].get_data();
                let end_x = begin[0] + (end[0] - begin[0]) * subpercent;
                let end_y = begin[1] + (end[1] - begin[1]) * subpercent;
                position = [end_x, end_y];
            }
            if points[0].pen_down {
                line_list.push((start, position));
            }
            start = end;
            angle = end_angle;
        }
        CircleDrawData {
            line_list,
            position,
            angle,
        }
    }
}

struct RatatuiInternal {
    terminal: RefCell<Terminal<CrosstermBackend<Stdout>>>,
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
    title: String,
    popups: HashMap<PopupID, PopupData>,
    next_id: PopupID,
    bgcolor: Color,
    size: [f32; 2],
}

impl Drop for RatatuiInternal {
    fn drop(&mut self) {
        let mut stdout = std::io::stdout();
        let _ = execute!(
            stdout,
            crossterm::event::PopKeyboardEnhancementFlags,
            crossterm::event::DisableMouseCapture
        );
    }
}

impl RatatuiInternal {
    fn new(flags: &TurtleFlags) -> Self {
        let mut stdout = std::io::stdout();
        let _ = execute!(
            stdout,
            crossterm::event::PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            ),
            crossterm::event::EnableMouseCapture,
        );
        let mut this = Self {
            terminal: RefCell::new(ratatui::init()),
            last_id: TurtleID::default(),
            turtle: HashMap::new(),
            title: format!(" {} ", flags.title),
            popups: HashMap::new(),
            next_id: PopupID::new(0),
            bgcolor: Color::White,
            size: flags.size,
        };
        let _turtle = this.new_turtle();
        this
    }

    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(
            id,
            IndividualTurtle {
                has_new_cmd: true,
                ..Default::default()
            },
        );
        id
    }

    fn generate_popup(&mut self, popupdata: PopupData) {
        let id = self.next_id.get();
        self.popups.insert(id, popupdata);
    }
}

#[derive(Debug)]
enum RatatuiDrawCmd {
    Line(Line),
    Circle(Circle),
    Text {
        x: f32,
        y: f32,
        text: String,
        color: Color,
    },
}

impl RatatuiDrawCmd {
    fn line(start: (f64, f64), end: (f64, f64), color: Color) -> Self {
        Self::Line(Line::new(start.0, -start.1, end.0, -end.1, color))
    }

    fn circle(center: (f64, f64), radius: f64, color: Color) -> Self {
        Self::Circle(Circle {
            x: center.0,
            y: -center.1,
            radius,
            color,
        })
    }

    fn text<S: ToString>(pos: &Point<f32>, text: S, color: Color) -> Self {
        Self::Text {
            x: pos.x,
            y: -pos.y,
            text: text.to_string(),
            color,
        }
    }
}

impl RatatuiFramework {
    pub(crate) fn start(mut flags: TurtleFlags) {
        let func = flags.start_func.take();

        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let tui = RatatuiInternal::new(&flags);
        let mut rata = Self { tt, tui };
        let _ = rata.run();

        ratatui::restore();
    }

    fn run(&mut self) -> Result<Event, std::io::Error> {
        let tick_rate = Duration::from_millis(1000 / 60);
        let mut last_tick = Instant::now();
        loop {
            let size = {
                let mut term = self.tui.terminal.borrow_mut();
                if let Err(e) = term.draw(|frame| self.draw(frame)) {
                    break Err(e);
                }
                term.size().expect("could not get screen size")
            };
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            match event::poll(timeout) {
                Err(e) => break Err(e),
                Ok(true) => {
                    let event = event::read().expect("should not have failed");
                    match event {
                        Event::Key(key) => {
                            if self.handle_key_event(key) {
                                break Ok(event);
                            }
                        }
                        Event::Mouse(me) => match me.kind {
                            MouseEventKind::Down(_button) => {
                                // actual display: self.tui.size[0] and [1]
                                // mouse coordinates: me/size .row and .column
                                //
                                // coords = ((actual display) / (size coords)) * (mevent coords)

                                let mouse_x = me.column as f32 - size.width as f32 / 2.;
                                let mouse_y = me.row as f32 - size.height as f32 / 2.;

                                let x = (self.tui.size[0] * mouse_x) / size.width as f32;
                                let y = (self.tui.size[1] * mouse_y) / size.height as f32;

                                let _ = self.tt.handle_event(
                                    None,
                                    None,
                                    &TurtleEvent::MousePress(x, -y),
                                );
                            }
                            MouseEventKind::Up(_button) => {}
                            MouseEventKind::Drag(_button) => {}
                            MouseEventKind::Moved => {}
                            _ => {}
                        },
                        Event::FocusGained
                        | Event::FocusLost
                        | Event::Paste(_)
                        | Event::Resize(_, _) => {
                            // We "resize" the window by scaling the drawing to the current
                            // physical window size. So if the user changes the size of the
                            // window, we scale things up or down as appropriate.
                        }
                    }
                }
                Ok(false) => {}
            }

            if last_tick.elapsed() >= tick_rate {
                self.tt.tick(&mut self.tui);

                // let mut done = true;
                for (tid, turtle) in &mut self.tui.turtle {
                    let (pct, prog) = self.tt.progress(*tid);
                    if turtle.has_new_cmd {
                        // done = false;
                        turtle.convert(pct);
                        if prog.is_done(pct) {
                            turtle.has_new_cmd = false;
                        }
                    }
                }
            }
            last_tick = Instant::now();
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(ch) => {
                // Ctrl-Q will exit the program no matter what
                if ch == 'q' && (key.modifiers & KeyModifiers::CONTROL) == KeyModifiers::CONTROL {
                    return true;
                }
                if self.tui.popups.is_empty() {
                    let e = if matches!(key.kind, KeyEventKind::Press) {
                        TurtleEvent::KeyPress(ch)
                    } else {
                        println!("YYYY");
                        TurtleEvent::KeyRelease(ch)
                    };
                    self.tt.handle_event(None, None, &e);
                } else {
                    for popup in self.tui.popups.values_mut() {
                        if popup.get_error().is_none() {
                            popup.get_text_mut().push(ch);
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                for popup in self.tui.popups.values_mut() {
                    popup.get_text_mut().pop();
                }
            }
            KeyCode::Esc => {
                for (_, popup) in self.tui.popups.drain() {
                    self.tt.popup_cancelled(popup.turtle(), popup.thread());
                }
            }
            KeyCode::Enter => {
                let mut new_popups = HashMap::new();
                for (key, mut popup) in self.tui.popups.drain() {
                    // Is there an error state we need to deal with?
                    if popup.get_error().is_some() {
                        popup.clear_error();
                        new_popups.insert(key, popup);
                    } else {
                        match popup.get_response() {
                            Ok(response) => {
                                self.tt
                                    .popup_result(popup.turtle(), popup.thread(), response);
                            }
                            Err(message) => {
                                popup.set_error(message);
                                popup.get_text_mut().clear();
                                new_popups.insert(key, popup);
                            }
                        }
                    }
                }
                self.tui.popups = new_popups;
            }
            _ => return true,
        }
        false
    }

    fn draw(&self, frame: &mut Frame) {
        let text_list_cmds = RefCell::new(Vec::new()); // TODO: can we do this without a RefCell?
        let width = self.tui.size[0];
        let height = self.tui.size[1];

        let x_bounds = [-(width / 2.) as f64, (width / 2.) as f64];
        let y_bounds = [-(height / 2.) as f64, (height / 2.) as f64];

        let area = frame.area();
        let widget = Canvas::default()
            .background_color(self.tui.bgcolor)
            .block(Block::bordered().title(self.tui.title.clone()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                for turtle in self.tui.turtle.values() {
                    let text_list = turtle.draw(ctx);
                    text_list_cmds.borrow_mut().extend(text_list);
                }
            })
            .x_bounds(x_bounds)
            .y_bounds(y_bounds);

        frame.render_widget(widget, area);

        for (x, y, sref, &cref) in text_list_cmds.borrow().iter() {
            let block = Block::new()
                .borders(Borders::NONE)
                .title((*sref).clone())
                .style(Style::new().fg(cref));
            let text_rect = Rect::new(*x as u16, *y as u16, sref.len() as u16, 1);
            frame.render_widget(block, text_rect);
        }

        for popup in self.tui.popups.values() {
            // /- TITLE -------------\
            // | <prompt>            |
            // | [<input-text>      ]|
            // \---------------------/

            let (has_err, prompt) = if let Some(err) = popup.get_error() {
                (true, err.as_str())
            } else {
                (false, popup.prompt())
            };
            let text = popup.get_text().to_string();
            let width = 25.max(popup.title().len().max(prompt.len())) + 2;
            let popup_area = Rect::new(10, 4, width.clamp_to(), 4);
            let entry_width = width - 4;
            let (entry, entry_len) = if text.len() < entry_width {
                (format!("[{:entry_width$}]", text.as_str()), text.len())
            } else {
                let shrink = entry_width - 2;
                (
                    format!("[..{:shrink$}]", &text[(text.len() - entry_width + 3)..]),
                    entry_width - 1,
                )
            };

            let popup = if has_err {
                Paragraph::new(vec![prompt.into(), TextLine::from("[ OK ]").centered()])
                    .block(Block::bordered().title(popup.title()))
                    .style(Style::new().fg(Color::Black).bg(Color::White))
            } else {
                frame.set_cursor_position(Position::new(
                    popup_area.x + 2 + entry_len.clamp_to_u16(),
                    popup_area.y + 2,
                ));
                Paragraph::new(vec![prompt.into(), entry.into()])
                    .block(
                        Block::bordered()
                            .title(popup.title())
                            .title_bottom(TextLine::from("Enter=OK").left_aligned())
                            .title_bottom(TextLine::from("Esc=Cancel").right_aligned()),
                    )
                    .style(Style::new().fg(Color::Black).bg(Color::White))
            };

            frame.render_widget(popup, popup_area);
        }
    }
}

impl TurtleGui for RatatuiInternal {
    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(
            id,
            IndividualTurtle {
                has_new_cmd: true,
                ..Default::default()
            },
        );
        id
    }

    fn shut_down(&mut self) {
        ratatui::restore();
        std::process::exit(0);
    }

    fn clear_turtle(&mut self, _turtle: TurtleID) {
        todo!()
    }

    fn set_shape(&mut self, turtle: TurtleID, shape: crate::polygon::TurtleShape) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape = shape;
        turtle.has_new_cmd = true;
    }

    fn stamp(&mut self, turtle: TurtleID, pos: crate::ScreenPosition<f32>, angle: f32) -> usize {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(DrawCommand::DrawPolyAt(
            turtle.turtle_shape.poly[0].polygon.clone(),
            pos,
            angle,
        ));
        turtle.has_new_cmd = true;
        turtle.cmds.len() - 1
    }

    fn clear_stamp(&mut self, turtle: TurtleID, stamp: usize) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        assert!(matches!(
            turtle.cmds[stamp],
            DrawCommand::DrawPolyAt(_, _, _)
        ));
        turtle.cmds[stamp] = DrawCommand::Filler;
        turtle.has_new_cmd = true;
    }

    fn clear_stamps(&mut self, turtle: TurtleID, count: StampCount) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        let all = turtle.cmds.len();
        let (mut iter, mut count) = match count {
            StampCount::Forward(count) => (Either::Right(turtle.cmds.iter_mut()), count),
            StampCount::Reverse(count) => (Either::Left(turtle.cmds.iter_mut().rev()), count),
            StampCount::All => (Either::Right(turtle.cmds.iter_mut()), all),
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if matches!(cmd, DrawCommand::DrawPolyAt(_, _, _)) {
                    count -= 1;
                    *cmd = DrawCommand::Filler;
                }
            } else {
                break;
            }
        }

        turtle.has_new_cmd = true;
    }

    fn get_turtle_shape_name(&mut self, turtle: TurtleID) -> String {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape.name.clone()
    }

    fn append_command(&mut self, turtle: TurtleID, cmd: DrawCommand) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(cmd);
        turtle.has_new_cmd = true;
    }

    fn get_position(&self, turtle: TurtleID) -> usize {
        self.turtle[&turtle].cmds.len()
    }

    fn fill_polygon(&mut self, turtle: TurtleID, cmd: DrawCommand, index: usize) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds[index] = cmd;
        turtle.cmds.push(DrawCommand::Filled(index));
        turtle.has_new_cmd = true;
    }

    fn undo(&mut self, turtle: TurtleID) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.has_new_cmd = true;
    }

    fn pop(&mut self, turtle: TurtleID) -> Option<DrawCommand> {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        let cmd = turtle.cmds.pop();

        if let Some(DrawCommand::Filled(index)) = &cmd {
            turtle.cmds[*index] = DrawCommand::Filler;
        }

        cmd
    }

    fn undo_count(&self, _turtle: TurtleID) -> usize {
        todo!()
    }

    fn numinput(
        &mut self,
        turtle: TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        self.generate_popup(PopupData::num_input(title, prompt, turtle, thread));
    }

    fn textinput(
        &mut self,
        turtle: TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        self.generate_popup(PopupData::text_input(title, prompt, turtle, thread));
    }

    fn bgcolor(&mut self, color: crate::color_names::TurtleColor) {
        self.bgcolor = color.into();
    }

    fn resize(
        &mut self,
        _turtle: TurtleID,
        _thread: crate::turtle::types::TurtleThread,
        width: isize,
        height: isize,
    ) {
        self.size = [width as f32, height as f32];
    }

    fn set_visible(&mut self, _turtle: TurtleID, _visible: bool) {
        todo!()
    }

    fn is_visible(&self, _turtle: TurtleID) -> bool {
        todo!()
    }

    fn clearscreen(&mut self) {
        todo!()
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
//
//TODO: use tryfrom instead?
impl From<&TurtleColor> for Color {
    fn from(value: &TurtleColor) -> Self {
        if let TurtleColor::Color(r, g, b) = value {
            Color::Rgb(
                (*r * 255.).clamp_to(),
                (*g * 255.).clamp_to(),
                (*b * 255.).clamp_to(),
            )
        } else {
            todo!()
        }
    }
}

impl From<TurtleColor> for Color {
    fn from(value: TurtleColor) -> Self {
        (&value).into()
    }
}

impl PolygonPath {
    // This code has been adapted from the example
    // in the lyon_tesselation docs.
    // See https://docs.rs/lyon_tessellation/latest/lyon_tessellation/struct.FillTessellator.html
    fn get_path(&self) -> Vec<(Point<f32>, Point<f32>)> {
        let mut path_builder = Path::builder();
        let mut iter = self.path.iter();
        let p = iter.next().expect("needs at least one point");
        path_builder.begin(point(p[0], p[1]));
        for p in iter {
            path_builder.line_to(point(p[0], p[1]));
        }
        path_builder.end(true);
        let path = path_builder.build();
        let mut buffers: VertexBuffers<Point<f32>, u16> = VertexBuffers::new();
        {
            let mut vertex_builder = simple_builder(&mut buffers);
            let mut tessellator = FillTessellator::new();
            tessellator
                .tessellate_path(&path, &FillOptions::default(), &mut vertex_builder)
                .expect("tesselation failed");
        }

        let mut result = Vec::new();

        for triangle in buffers.indices.as_slice().chunks(3) {
            let p0 = triangle[0] as usize;
            let p1 = triangle[1] as usize;
            let p2 = triangle[2] as usize;
            result.push((buffers.vertices[p0], buffers.vertices[p1]));
            result.push((buffers.vertices[p1], buffers.vertices[p2]));
            result.push((buffers.vertices[p2], buffers.vertices[p0]));
        }

        result
    }
}

/*
 * This is a Rust interpretation of the triangle fill algorithm taken from
 * Gabriel Gambetta: https://gabrielgambetta.com/computer-graphics-from-scratch/07-filled-triangles.html
 */
fn get_fill_lines(points: &[(Point<f32>, Point<f32>)]) -> Vec<(Point<f32>, Point<f32>)> {
    let mut triangle = [points[0].0, points[1].0, points[2].0];
    if triangle[1].y < triangle[0].y {
        triangle.swap(0, 1);
    }
    if triangle[2].y < triangle[0].y {
        triangle.swap(0, 2);
    }
    if triangle[2].y < triangle[1].y {
        triangle.swap(2, 1);
    }

    let mut x01 = interpolate(triangle[0].y, triangle[0].x, triangle[1].y, triangle[1].x);
    let x12 = interpolate(triangle[1].y, triangle[1].x, triangle[2].y, triangle[2].x);
    let x02 = interpolate(triangle[0].y, triangle[0].x, triangle[2].y, triangle[2].x);
    x01.pop();
    x01.extend(x12);

    let m = x01.len() / 2;
    let (x_left, x_right) = if x02[m] < x01[m] {
        (x02, x01)
    } else {
        (x01, x02)
    };

    let y0 = triangle[0].y as isize;

    (0..x_left.len())
        .map(|idx| {
            (
                Point::new(x_left[idx], (y0 + idx as isize) as f32),
                Point::new(x_right[idx], (y0 + idx as isize) as f32),
            )
        })
        .collect()
}

fn interpolate(i0: f32, d0: f32, i1: f32, d1: f32) -> Vec<f32> {
    if i0 == i1 {
        return vec![d0];
    }
    let mut values = Vec::new();
    let a = (d1 - d0) / (i1 - i0);
    let mut d = d0;
    let mut i = i0;
    while i <= i1 {
        values.push(d);
        d += a;
        i += 1.;
    }
    values
}
