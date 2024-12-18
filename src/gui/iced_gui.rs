#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::collections::HashMap;

use iced::widget::text;

use iced::{
    alignment::{Horizontal, Vertical},
    event, executor, mouse,
    multi_window::Application,
    widget::{
        button,
        canvas::{self, fill::Rule, stroke, Cache, Fill, Frame, LineJoin, Path, Stroke, Text},
        column, container, horizontal_space, row,
        text::{LineHeight, Shaping},
        text_input, vertical_space, Canvas, TextInput,
    },
    window::{self, Id as WindowID},
    Color, Element, Event, Font, Length, Point, Rectangle, Renderer, Settings, Size, Subscription,
    Theme,
};

use iced::keyboard::{Event::KeyPressed, Event::KeyReleased, Key};
use iced::window::Event::Resized;

use either::Either;
use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};

use super::{events::TurtleEvent, StampCount};
use crate::generate::{CirclePos, LineInfo};
use crate::polygon::PolygonPath;
use crate::{
    color_names::TurtleColor,
    generate::DrawCommand,
    gui::{popup::PopupData, TurtleGui},
    polygon::TurtleShape,
    turtle::{
        task::{EventResult, TurtleTask},
        types::{TurtleID, TurtleThread},
        TurtleFlags,
    },
    ScreenPosition,
};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Tick,
    Event(Event),
    TextInputChanged(WindowID, String),
    TextInputSubmit(WindowID),
    AckError(WindowID),
    Cancel(WindowID),
}

#[derive(Debug)]
pub(crate) enum IcedDrawCmd {
    Stroke(Path, Color, f32),
    Fill(Path, Color),
    Text(Point, String),
}

#[derive(Default)]
struct IndividualTurtle {
    cmds: Vec<DrawCommand>,
    drawing: Vec<IcedDrawCmd>,
    has_new_cmd: bool,
    turtle_shape: TurtleShape,
    hide_turtle: bool,
}

impl IndividualTurtle {
    fn draw(&self, frame: &mut Frame) {
        for draw_iced_cmd in &self.drawing {
            match draw_iced_cmd {
                IcedDrawCmd::Stroke(path, pencolor, penwidth) => frame.stroke(
                    path,
                    Stroke {
                        style: stroke::Style::Solid(*pencolor),
                        width: *penwidth,
                        line_join: LineJoin::Round,
                        ..Stroke::default()
                    },
                ),
                IcedDrawCmd::Fill(path, fillcolor) => frame.fill(
                    path,
                    Fill {
                        style: stroke::Style::Solid(*fillcolor),
                        rule: Rule::EvenOdd,
                    },
                ),
                IcedDrawCmd::Text(pos, text) => {
                    frame.fill_text(Text {
                        content: text.to_string(),
                        position: *pos,
                        color: Color::BLACK,
                        size: 10.into(),
                        line_height: LineHeight::Relative(1.0),
                        font: Font::DEFAULT,
                        horizontal_alignment: Horizontal::Left,
                        vertical_alignment: Vertical::Bottom,
                        shaping: Shaping::Basic,
                    });
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn convert(&mut self, pct: f32) {
        fn make_path(path: &mut Vec<(bool, Point)>) -> Path {
            Path::new(|b| {
                b.move_to(path[0].1);
                for (pen, pos) in path.drain(1..) {
                    if pen {
                        b.line_to(pos);
                    } else {
                        b.move_to(pos);
                    }
                }
                path.clear(); // remove first element
            })
        }

        let mut pencolor = Color::BLACK;
        let mut penwidth = 1.0;
        let mut fillcolor = Color::BLACK;

        let mut tpos = [0f32, 0f32];
        let mut trot = 0f32;

        self.drawing.clear();

        let mut iter = self.cmds.iter().peekable();
        let mut cur_path: Vec<(bool, Point)> = Vec::new();

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            if !matches!(element, DrawCommand::Line(..))
                && !matches!(element, DrawCommand::SetHeading(..))
                && !cur_path.is_empty()
            {
                self.drawing.push(IcedDrawCmd::Stroke(
                    make_path(&mut cur_path),
                    pencolor,
                    penwidth,
                ));
            }

            match element {
                DrawCommand::Line(line) => {
                    let (start, end) = Self::start_and_end(last_element, pct, line);
                    tpos = [end.x, end.y];
                    if cur_path.is_empty() {
                        cur_path.push((line.pen_down, start));
                    }
                    cur_path.push((line.pen_down, end));
                }
                DrawCommand::SetPenColor(pc) => {
                    pencolor = pc.into();
                }
                DrawCommand::SetPenWidth(pw) => penwidth = *pw,
                DrawCommand::SetFillColor(fc) => {
                    fillcolor = fc.into();
                }
                DrawCommand::DrawPolygon(p) => {
                    self.drawing
                        .push(IcedDrawCmd::Fill(p.get_path().clone(), fillcolor));
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
                    let center: Point = Point::new(center.x, center.y);
                    let circle = Path::circle(center, *radius);
                    self.drawing.push(IcedDrawCmd::Fill(circle, color.into()));
                }
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let path = polygon.get_path();
                    let angle = Angle::degrees(*angle);
                    let xform = Transform2D::rotation(angle).then_translate([pos.x, pos.y].into());
                    let path = path.transform(&xform);
                    self.drawing
                        .push(IcedDrawCmd::Fill(path.clone(), fillcolor));
                    self.drawing
                        .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
                }
                DrawCommand::Circle(points) => {
                    let (path, final_pos, final_angle) =
                        Self::circle_path(last_element, pct, points);
                    tpos = final_pos.into();
                    trot = final_angle;
                    self.drawing
                        .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
                }
                DrawCommand::SetPosition(pos) => {
                    tpos = [pos.x as f32, pos.y as f32];
                }
                DrawCommand::Text(pos, text) => {
                    let pos = Point { x: pos.x, y: pos.y };
                    self.drawing.push(IcedDrawCmd::Text(pos, text.to_string()));
                }
                DrawCommand::Filler | DrawCommand::Filled(_) => {}
                DrawCommand::StampTurtle
                | DrawCommand::Clear
                | DrawCommand::Reset
                | DrawCommand::BeginFill
                | DrawCommand::EndFill
                | DrawCommand::BeginPoly
                | DrawCommand::EndPoly => panic!("invalid draw command in gui"),
            }
        }

        if !cur_path.is_empty() {
            self.drawing.push(IcedDrawCmd::Stroke(
                make_path(&mut cur_path),
                pencolor,
                penwidth,
            ));
        }

        if !self.hide_turtle {
            for (path, fillcolor, pencolor) in
                self.calculate_turtle(tpos, trot, (&fillcolor).into(), (&pencolor).into())
            {
                self.drawing
                    .push(IcedDrawCmd::Fill(path.clone(), (&fillcolor).into()));
                self.drawing
                    .push(IcedDrawCmd::Stroke(path, (&pencolor).into(), penwidth));
            }
        }
    }

    fn calculate_turtle(
        &self,
        tpos: [f32; 2],
        trot: f32,
        fillcolor: TurtleColor,
        pencolor: TurtleColor,
    ) -> Vec<(Path, TurtleColor, TurtleColor)> {
        let angle = Angle::degrees(trot);
        let transform = Transform2D::rotation(angle).then_translate(tpos.into());
        let mut result = Vec::new();

        for poly in &self.turtle_shape.poly {
            let path = poly.polygon.get_path();
            let path = path.transform(&transform);

            let fillcolor = fillcolor.color_or(&poly.fill);
            let pencolor = pencolor.color_or(&poly.outline);
            result.push((path, fillcolor, pencolor));
        }

        result
    }

    fn start_and_end(last_element: bool, pct: f32, line: &LineInfo) -> (Point, Point) {
        (
            [line.begin.x as f32, line.begin.y as f32].into(),
            if last_element {
                let end_x = line.begin.x as f32 + (line.end.x - line.begin.x) as f32 * pct;
                let end_y = line.begin.y as f32 + (line.end.y - line.begin.y) as f32 * pct;
                [end_x, end_y]
            } else {
                [line.end.x as f32, line.end.y as f32]
            }
            .into(),
        )
    }

    // returns path, final point, and final angle
    fn circle_path(last_element: bool, pct: f32, points: &[CirclePos]) -> (Path, Point, f32) {
        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut tpos = Point::default();
        let mut trot = 0.;
        let path = Path::new(|b| {
            let (_, start) = points[0].get_data();

            b.move_to(start.into());

            let mut iter = points.windows(2).take(total + 1).peekable();
            while let Some(p) = iter.next() {
                let (end_angle, end) = p[1].get_data();
                let last_segment = iter.peek().is_none();
                tpos = end.into();
                if last_element && last_segment {
                    let (_, begin) = p[0].get_data();
                    let end_x = begin[0] + (end[0] - begin[0]) * subpercent;
                    let end_y = begin[1] + (end[1] - begin[1]) * subpercent;
                    tpos = [end_x, end_y].into();
                }
                if points[0].pen_down {
                    b.line_to(tpos);
                } else {
                    b.move_to(tpos);
                }
                trot = end_angle;
            }
        });
        (path, tpos, trot)
    }
}

type IcedCommand<T> = iced::Command<T>;

pub(crate) struct IcedGuiFramework {
    cache: Cache,
    tt: TurtleTask,
    gui: IcedGuiInternal,
    clear_cache: bool,
    winsize: (f32, f32),   // width, height
    mouse_pos: (f32, f32), // x, y
    mouse_down: bool,
}

#[derive(Default)]
struct IcedGuiInternal {
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
    popups: HashMap<WindowID, PopupData>,
    title: String,
    wcmds: Vec<IcedCommand<Message>>,
    bgcolor: TurtleColor,
    resize_request: Option<(TurtleID, TurtleThread)>,
}

impl TurtleGui for IcedGuiInternal {
    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn clearscreen(&mut self) {
        let id0 = TurtleID::new(0);

        self.turtle.retain(|&k, _| k == id0);

        self.turtle.entry(id0).and_modify(|t| {
            *t = IndividualTurtle {
                has_new_cmd: true,
                ..Default::default()
            }
        });
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

    fn shut_down(&mut self) {
        std::process::exit(0);
    }

    fn clear_turtle(&mut self, turtle: TurtleID) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.clear();
        turtle.has_new_cmd = true;
    }

    fn set_shape(&mut self, turtle: TurtleID, shape: TurtleShape) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape = shape;
        turtle.has_new_cmd = true;
    }

    fn stamp(&mut self, turtle: TurtleID, pos: ScreenPosition<f32>, angle: f32) -> usize {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(DrawCommand::DrawPolyAt(
            turtle.turtle_shape.poly[0].polygon.clone(),
            pos,
            angle,
        ));
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
        turtle.has_new_cmd = true;
        turtle.cmds[index] = cmd;
        turtle.cmds.push(DrawCommand::Filled(index));
    }

    fn undo_count(&self, turtle: TurtleID) -> usize {
        self.turtle.get(&turtle).expect("missing turtle").cmds.len()
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

    fn numinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        self.generate_popup(PopupData::num_input(title, prompt, turtle, thread));
    }

    fn textinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        self.generate_popup(PopupData::text_input(title, prompt, turtle, thread));
    }

    fn bgcolor(&mut self, color: TurtleColor) {
        self.bgcolor = color;
    }

    fn resize(&mut self, turtle: TurtleID, thread: TurtleThread, width: isize, height: isize) {
        let new_size = Size::new(width as f32, height as f32);
        self.wcmds
            .push(window::resize::<Message>(window::Id::MAIN, new_size));
        self.resize_request = Some((turtle, thread));
    }

    fn set_visible(&mut self, turtle: TurtleID, visible: bool) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.hide_turtle = !visible;
        turtle.has_new_cmd = true;
    }

    fn is_visible(&self, turtle: TurtleID) -> bool {
        let turtle = self.turtle.get(&turtle).expect("missing turtle");
        !turtle.hide_turtle
    }
}

impl Application for IcedGuiFramework {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TurtleFlags;

    fn new(mut flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let func = flags.start_func.take();

        let title = flags.title.clone();
        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let framework = Self {
            cache: Cache::default(),
            tt,
            clear_cache: true,
            gui: IcedGuiInternal::new(title),
            winsize: (0., 0.),
            mouse_pos: (0., 0.),
            mouse_down: false,
        };

        (framework, IcedCommand::none())
    }

    fn title(&self, win_id: iced::window::Id) -> String {
        if win_id == WindowID::MAIN {
            self.gui.title.clone()
        } else {
            self.gui
                .popups
                .get(&win_id)
                .expect("lookup popup data")
                .title()
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Tick => {
                if self.clear_cache {
                    self.cache.clear();
                    self.clear_cache = false;
                }
                self.tt.tick(&mut self.gui);
                if self.update_turtles() {
                    self.clear_cache = true;
                }
            }
            Message::AckError(id) => {
                let popup = self.gui.popups.get_mut(&id).expect("looking up popup data");
                popup.clear_error();
            }
            Message::Event(event) => {
                let turtle_event: TurtleEvent = event.into();
                match &turtle_event {
                    TurtleEvent::WindowResize(x, y) => {
                        self.winsize = (*x as f32, *y as f32);
                        if self.gui.resize_request.is_none() {
                            self.tt.handle_event(None, None, &turtle_event);
                        } else {
                            let (turtle, thread) =
                                self.gui.resize_request.expect("missing resize data");
                            self.tt
                                .handle_event(Some(turtle), Some(thread), &turtle_event);
                        }
                    }
                    TurtleEvent::MousePosition(x, y) => {
                        self.mouse_pos = self.to_turtle_pos(*x, *y);
                        if self.mouse_down {
                            self.tt.handle_event(
                                None,
                                None,
                                &TurtleEvent::MouseDrag(self.mouse_pos.0, self.mouse_pos.1),
                            );
                        }
                    }
                    TurtleEvent::MouseDrag(_, _) => unimplemented!(),
                    TurtleEvent::MousePress(_x, _y) => {
                        self.mouse_down = true;
                        if self.tt.handle_event(
                            None,
                            None,
                            &TurtleEvent::MousePress(self.mouse_pos.0, self.mouse_pos.1),
                        ) == EventResult::ShutDown
                        {
                            std::process::exit(0);
                        }
                    }
                    TurtleEvent::MouseRelease(_x, _y) => {
                        self.mouse_down = false;
                        self.tt.handle_event(
                            None,
                            None,
                            &TurtleEvent::MouseRelease(self.mouse_pos.0, self.mouse_pos.1),
                        );
                    }
                    TurtleEvent::Unhandled => {}
                    TurtleEvent::KeyPress(_) | TurtleEvent::KeyRelease(_) => {
                        self.tt.handle_event(None, None, &turtle_event);
                    }
                    TurtleEvent::_Timer => todo!(),
                }
            }
            Message::TextInputChanged(id, msg) => {
                let popup = self.gui.popups.get_mut(&id).expect("looking up popup data");
                popup.set_message(&msg);
            }
            Message::TextInputSubmit(id) => {
                let mut popup = self.gui.popups.remove(&id).expect("looking up popup data");
                match popup.get_response() {
                    Ok(response) => {
                        let turtle = popup.turtle();
                        let thread = popup.thread();
                        self.tt.popup_result(turtle, thread, response);
                        self.gui.wcmds.push(window::close(id));
                    }
                    Err(message) => {
                        popup.set_error(message);
                        self.gui.popups.insert(id, popup);
                    }
                }
            }
            Message::Cancel(id) => {
                let popup = self.gui.popups.remove(&id).expect("looking up popup data");
                self.tt.popup_cancelled(popup.turtle(), popup.thread());
                self.gui.wcmds.push(window::close(id));
            }
        }
        IcedCommand::batch(self.gui.wcmds.drain(..).collect::<Vec<_>>())
    }

    fn view(
        &self,
        win_id: iced::window::Id,
    ) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        if win_id == WindowID::MAIN {
            Canvas::new(self)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            let popup = self
                .gui
                .popups
                .get(&win_id)
                .expect("looking up window data");
            if let Some(error) = popup.get_error() {
                container(
                    column![
                        vertical_space(),
                        row![horizontal_space(), text(error), horizontal_space()],
                        vertical_space(),
                        row![
                            horizontal_space(),
                            button("OK").on_press(Message::AckError(win_id)),
                            horizontal_space(),
                        ],
                        vertical_space(),
                    ]
                    .width(200),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
            } else {
                let prompt = popup.prompt();
                let text_field: TextInput<Self::Message> = text_input(prompt, popup.get_text())
                    .width(200)
                    .on_input(move |msg| Message::TextInputChanged(win_id, msg))
                    .on_submit(Message::TextInputSubmit(win_id));
                let data: Element<Self::Message> = container(row![
                    horizontal_space(),
                    column![text(prompt), text_field],
                    horizontal_space(),
                ])
                .center_x()
                .into();
                let buttons: Element<Self::Message> = container(row![
                    horizontal_space(),
                    button("Cancel").on_press(Message::Cancel(win_id)),
                    horizontal_space(),
                    button("OK").on_press(Message::TextInputSubmit(win_id)),
                    horizontal_space(),
                ])
                .padding(10)
                .into();
                container(column![data, buttons])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick),
            event::listen().map(Message::Event),
        ])
    }
}

impl<Message> canvas::Program<Message> for IcedGuiFramework {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<<Renderer as canvas::Renderer>::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            frame.fill_rectangle(
                [0., 0.].into(),
                bounds.size(),
                Fill {
                    style: stroke::Style::Solid((&self.gui.bgcolor).into()),
                    rule: Rule::NonZero,
                },
            );
            frame.translate([center.x, center.y].into());
            for turtle in self.gui.turtle.values() {
                turtle.draw(frame);
            }
        });
        vec![geometry]
    }
}

impl IcedGuiFramework {
    pub(crate) fn start(flags: TurtleFlags) {
        let (xsize, ysize) = (flags.size[0], flags.size[1]);

        Self::run(Settings {
            antialiasing: true,
            flags,
            window: window::Settings {
                size: Size::new(xsize, ysize),
                ..Default::default()
            },
            ..Settings::default()
        })
        .expect("failed to start turtle");
    }

    // returns true if the cache should be cleared
    fn update_turtles(&mut self) -> bool {
        let mut done = true;

        for (tid, turtle) in &mut self.gui.turtle {
            let (pct, prog) = self.tt.progress(*tid);
            if turtle.has_new_cmd {
                done = false;
                turtle.convert(pct);
                if prog.is_done(pct) {
                    turtle.has_new_cmd = false;
                }
            }
        }

        !done
    }

    fn to_turtle_pos(&self, x: f32, y: f32) -> (f32, f32) {
        (x - self.winsize.0 / 2., -(y - self.winsize.1 / 2.))
    }
}

impl IcedGuiInternal {
    fn new(title: String) -> Self {
        let mut this = Self {
            popups: HashMap::new(),
            title,
            bgcolor: TurtleColor::from("white"),
            ..Self::default()
        };
        let _turtle = this.new_turtle();
        this
    }

    fn generate_popup(&mut self, popupdata: PopupData) {
        let (id, wcmd) = window::spawn(window::Settings {
            size: [250f32, 150f32].into(),
            resizable: false,
            exit_on_close_request: false,
            ..window::Settings::default()
        });
        self.wcmds.push(wcmd);
        self.popups.insert(id, popupdata);
    }
}

impl From<Event> for TurtleEvent {
    fn from(event: Event) -> Self {
        fn convert_mouse_event(event: mouse::Event) -> TurtleEvent {
            match event {
                mouse::Event::CursorMoved { position } => {
                    TurtleEvent::MousePosition(position.x, position.y)
                }
                mouse::Event::ButtonPressed(_) => TurtleEvent::MousePress(0., 0.),
                mouse::Event::ButtonReleased(_) => TurtleEvent::MouseRelease(0., 0.),
                _ => TurtleEvent::Unhandled,
            }
        }

        match event {
            Event::Keyboard(KeyReleased { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    TurtleEvent::KeyRelease(ch)
                } else {
                    TurtleEvent::Unhandled
                }
            }
            Event::Keyboard(KeyPressed { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    TurtleEvent::KeyPress(ch)
                } else {
                    TurtleEvent::Unhandled
                }
            }
            Event::Window(window::Id::MAIN, Resized { width, height }) =>
            {
                #[allow(clippy::cast_possible_wrap)]
                TurtleEvent::WindowResize(width as isize, height as isize)
            }
            Event::Mouse(mouse_event) => convert_mouse_event(mouse_event),
            Event::Touch(_) | Event::Window(..) | Event::Keyboard(_) => TurtleEvent::Unhandled,
        }
    }
}

impl PolygonPath {
    fn get_path(&self) -> Path {
        let mut iter = self.path.iter();
        let first = iter.next().unwrap();
        Path::new(|b| {
            b.move_to((*first).into());
            for i in iter {
                b.line_to((*i).into());
            }
        })
    }
}

//TODO: use tryfrom instead?
impl From<&TurtleColor> for iced::Color {
    fn from(value: &TurtleColor) -> Self {
        if let TurtleColor::Color(r, g, b) = value {
            iced::Color {
                r: *r,
                g: *g,
                b: *b,
                a: 1.,
            }
        } else {
            todo!()
        }
    }
}

impl From<&iced::Color> for TurtleColor {
    fn from(value: &iced::Color) -> Self {
        TurtleColor::Color(value.r, value.g, value.b)
    }
}
