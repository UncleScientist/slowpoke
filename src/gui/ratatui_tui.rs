use std::{
    cell::RefCell,
    collections::HashMap,
    io::Stdout,
    time::{Duration, Instant},
};

use either::Either;
use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::event::{self, Event},
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Circle, Context, Line},
        Block,
    },
    Frame, Terminal,
};

use crate::{
    color_names::TurtleColor,
    generate::{CirclePos, DrawCommand},
    polygon::TurtleShape,
    turtle::{task::TurtleTask, types::TurtleID, TurtleFlags},
};

use super::{StampCount, TurtleGui};

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

impl IndividualTurtle {
    fn draw(&self, ctx: &mut Context) {
        for cmd in &self.drawing {
            match cmd {
                RatatuiDrawCmd::Line(l) => ctx.draw(l),
                RatatuiDrawCmd::Circle(c) => ctx.draw(c),
                RatatuiDrawCmd::Text { x, y, text } => todo!(),
            }
        }
    }

    fn convert(&mut self, pct: f32) {
        let mut pencolor = TurtleColor::default();
        let mut trot = 0f32;
        let mut tpos = [0f64, 0f64];
        let mut iter = self.cmds.iter().peekable();

        self.drawing.clear();

        while let Some(cmd) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            match cmd {
                DrawCommand::Line(l) => {
                    let (begin_x, begin_y): (f64, f64) = (l.begin.x as f64, l.begin.y as f64);
                    let (end_x, end_y) = if last_element {
                        let end_x = l.begin.x as f32 + (l.end.x - l.begin.x) as f32 * pct;
                        let end_y = l.begin.y as f32 + (l.end.y - l.begin.y) as f32 * pct;
                        tpos = [end_x as f64, end_y as f64];
                        (tpos[0], tpos[1])
                    } else {
                        tpos = [l.end.x as f64, l.end.y as f64];
                        (tpos[0], tpos[1])
                    };
                    if l.pen_down {
                        self.drawing.push(RatatuiDrawCmd::Line(Line::new(
                            begin_x,
                            begin_y,
                            end_x,
                            end_y,
                            (&pencolor).into(),
                        )));
                    }
                }
                DrawCommand::Filler | DrawCommand::Filled(_) => {}
                DrawCommand::SetPenColor(pc) => pencolor = *pc,
                DrawCommand::SetPenWidth(_) => {
                    // TODO: figure out pen width for ratatui
                }
                DrawCommand::SetFillColor(_) => {
                    // TODO: figure out fill color for ratatui
                }
                DrawCommand::SetPosition(pos) => {
                    tpos = [pos.x as f64, pos.y as f64];
                }
                DrawCommand::DrawPolygon(_) => {
                    // TODO: How to fill a polygon in ratatui?
                }
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * pct
                    } else {
                        *end
                    };
                    trot = rotation;
                }
                DrawCommand::DrawDot(center, radius, color) => {
                    self.drawing.push(RatatuiDrawCmd::Circle(Circle {
                        x: center.x as f64,
                        y: center.y as f64,
                        radius: *radius as f64,
                        color: color.into(),
                    }));
                }
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let angle = Angle::degrees(*angle);
                    let pos = [pos.x, pos.y];
                    let transform = Transform2D::rotation(angle).then_translate(pos.into());

                    for pair in polygon.path.as_slice().windows(2) {
                        let p1 = pair[0];
                        let p2 = pair[1];
                        let start = transform.transform_point(p1.into());
                        let end = transform.transform_point(p2.into());
                        self.drawing.push(RatatuiDrawCmd::Line(Line::new(
                            start.x as f64,
                            start.y as f64,
                            end.x as f64,
                            end.y as f64,
                            (&pencolor).into(),
                        )));
                    }
                }
                DrawCommand::Circle(points) => {
                    let (line_list, final_pos, final_angle) =
                        Self::circle_path(last_element, pct, points);
                    tpos = [final_pos[0] as f64, final_pos[1] as f64];
                    trot = final_angle;
                    for line in line_list {
                        self.drawing.push(RatatuiDrawCmd::Line(Line::new(
                            line.0[0] as f64,
                            line.0[1] as f64,
                            line.1[0] as f64,
                            line.1[1] as f64,
                            (&pencolor).into(),
                        )));
                    }
                }
                DrawCommand::Text(_, _) => todo!(),
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

                    self.drawing.push(RatatuiDrawCmd::Line(Line::new(
                        start.x as f64,
                        start.y as f64,
                        end.x as f64,
                        end.y as f64,
                        (&pencolor).into(),
                    )));
                }
            }
        }
    }

    // returns path, final point, and final angle
    fn circle_path(
        last_element: bool,
        pct: f32,
        points: &[CirclePos],
    ) -> (Vec<([f32; 2], [f32; 2])>, [f32; 2], f32) {
        let mut line_list = Vec::new();

        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut tpos = [0., 0.];
        let mut trot = 0.;
        let (_, mut start) = points[0].get_data();

        let mut iter = points.windows(2).take(total + 1).peekable();
        while let Some(p) = iter.next() {
            let (end_angle, end) = p[1].get_data();
            let last_segment = iter.peek().is_none();
            tpos = end;
            if last_element && last_segment {
                let (_, begin) = p[0].get_data();
                let end_x = begin[0] + (end[0] - begin[0]) * subpercent;
                let end_y = begin[1] + (end[1] - begin[1]) * subpercent;
                tpos = [end_x, end_y];
            }
            if points[0].pen_down {
                line_list.push((start, tpos));
            }
            start = end;
            trot = end_angle;
        }
        (line_list, tpos, trot)
    }
}

struct RatatuiInternal {
    terminal: RefCell<Terminal<CrosstermBackend<Stdout>>>,
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
    title: String, // TODO: implement popups
}

impl RatatuiInternal {
    fn new() -> Self {
        let mut this = Self {
            terminal: RefCell::new(ratatui::init()),
            last_id: TurtleID::default(),
            turtle: HashMap::new(),
            title: "*default title*".to_string(),
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
}

enum RatatuiDrawCmd {
    Line(Line),
    Circle(Circle),
    Text { x: f32, y: f32, text: String },
}

impl RatatuiFramework {
    pub(crate) fn start(mut flags: TurtleFlags) {
        let func = flags.start_func.take();

        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let tui = RatatuiInternal::new();
        let mut rata = Self { tt, tui };
        let _ = rata.run();

        ratatui::restore();
    }

    fn run(&mut self) -> Result<Event, std::io::Error> {
        let tick_rate = Duration::from_millis(1000 / 60);
        let mut last_tick = Instant::now();
        loop {
            {
                let mut term = self.tui.terminal.borrow_mut();
                if let Err(e) = term.draw(|frame| self.draw(frame)) {
                    break Err(e);
                }
            }
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            match event::poll(timeout) {
                Err(e) => break Err(e),
                Ok(true) => {
                    let event = event::read().expect("should not have failed");
                    break Ok(event);
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

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let widget = Canvas::default()
            .background_color(Color::White)
            .block(Block::bordered().title(self.tui.title.clone()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                for (tid, turtle) in self.tui.turtle.iter() {
                    let (pct, _) = self.tt.progress(*tid);
                    turtle.draw(ctx);
                }
            })
            .x_bounds([-200., 200.])
            .y_bounds([-200., 200.]);

        frame.render_widget(widget, area);
        /*
         * Render pop-up windows here
        let overlay = Rect {
            x: 10,
            y: 10,
            width: 5,
            height: 3,
        };
        frame.render_widget(Block::bordered().title("a block"), overlay);
        */
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

    fn undo(&mut self, _turtle: TurtleID) {
        todo!()
    }

    fn pop(&mut self, _turtle: TurtleID) -> Option<DrawCommand> {
        todo!()
    }

    fn undo_count(&self, _turtle: TurtleID) -> usize {
        todo!()
    }

    fn numinput(
        &mut self,
        _turtle: TurtleID,
        _thread: crate::turtle::types::TurtleThread,
        _title: &str,
        _prompt: &str,
    ) {
        todo!()
    }

    fn textinput(
        &mut self,
        _turtle: TurtleID,
        _thread: crate::turtle::types::TurtleThread,
        _title: &str,
        _prompt: &str,
    ) {
        todo!()
    }

    fn bgcolor(&mut self, _color: crate::color_names::TurtleColor) {
        // TODO: change background color
    }

    fn resize(
        &mut self,
        _turtle: TurtleID,
        _thread: crate::turtle::types::TurtleThread,
        _width: isize,
        _height: isize,
    ) {
        todo!()
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
            Color::Rgb((*r * 255.) as u8, (*g * 255.) as u8, (*b * 255.) as u8)
        } else {
            todo!()
        }
    }
}
