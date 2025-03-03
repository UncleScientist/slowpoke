#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use iced::Pixels;
use slowpoke::{
    CirclePos, DrawCommand, EventResult, Handler, IndividualTurtle, LineInfo, PolygonPath,
    PopupData, PopupID, SlowpokeLib, TurtleColor, TurtleEvent, TurtleFlags, TurtleGui, TurtleID,
    TurtleTask, TurtleThread, TurtleUI, TurtleUserInterface,
};

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

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
    Color, Element, Font, Length, Point, Rectangle, Renderer, Settings, Size, Subscription, Theme,
};

use iced::keyboard::{Event::KeyPressed, Event::KeyReleased, Key};
use iced::window::Event::Resized;

use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};

pub type Slowpoke = SlowpokeLib<IcedGuiFramework>;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Event(Event),
    TextInputChanged(WindowID, String),
    TextInputSubmit(WindowID),
    AckError(WindowID),
    Cancel(WindowID),
}

#[derive(Debug)]
pub(crate) enum IcedDrawCmd {
    Stroke(Path, IcedColor, f32),
    Fill(Path, IcedColor),
    Text(Point, String),
}

#[derive(Default, Debug)]
struct IcedUI {
    drawing: Vec<IcedDrawCmd>,
}

impl IcedUI {
    fn draw(&self, frame: &mut Frame) {
        for draw_iced_cmd in &self.drawing {
            match draw_iced_cmd {
                IcedDrawCmd::Stroke(path, pencolor, penwidth) => frame.stroke(
                    path,
                    Stroke {
                        style: stroke::Style::Solid(pencolor.into()),
                        width: *penwidth,
                        line_join: LineJoin::Round,
                        ..Stroke::default()
                    },
                ),
                IcedDrawCmd::Fill(path, fillcolor) => frame.fill(
                    path,
                    Fill {
                        style: stroke::Style::Solid(fillcolor.into()),
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
    fn convert(&mut self, pct: f32, cmds: &[DrawCommand], turtle: &IndividualTurtle<IcedUI>) {
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

        let mut pencolor = IcedColor(Color::BLACK);
        let mut penwidth = 1.0;
        let mut fillcolor = IcedColor(Color::BLACK);

        let mut tpos = [0f32, 0f32];
        let mut trot = 0f32;

        self.drawing.clear();

        let mut iter = cmds.iter().peekable();
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
                DrawCommand::Dot(center, radius, color) => {
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

        if !turtle.hide_turtle {
            for (path, fillcolor, pencolor) in
                self.calculate_turtle(tpos, trot, fillcolor.into(), pencolor.into(), turtle)
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
        turtle: &IndividualTurtle<IcedUI>,
    ) -> Vec<(Path, TurtleColor, TurtleColor)> {
        let angle = Angle::degrees(trot);
        let transform = Transform2D::rotation(angle).then_translate(tpos.into());
        let mut result = Vec::new();

        for poly in &turtle.turtle_shape.poly {
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
type IcedWinId = iced::window::Id;

impl TurtleUI for IcedGuiInternal {
    fn generate_popup(&mut self, _popupdatata: &PopupData) -> PopupID {
        let (id, wcmd) = window::spawn(window::Settings {
            size: [250f32, 150f32].into(),
            resizable: false,
            exit_on_close_request: false,
            ..window::Settings::default()
        });
        self.wcmds.push(wcmd);
        let popup_id = self.next_id.get();
        self.winid_to_popupid.insert(id, popup_id);
        popup_id
    }

    fn resize(&mut self, width: isize, height: isize) {
        let new_size = Size::new(width as f32, height as f32);
        self.wcmds
            .push(window::resize::<Message>(window::Id::MAIN, new_size));
        // self.resize_request = Some((turtle, thread)); TODO
    }

    fn set_bg_color(&mut self, bgcolor: TurtleColor) {
        self.bgcolor = bgcolor;
    }
}

#[derive(Debug)]
pub struct IcedGuiFramework {
    cache: Cache,
    tt: TurtleTask,
    handler: Handler<IcedUI, IcedGuiInternal>,
    clear_cache: bool,
    winsize: (f32, f32),   // width, height
    mouse_pos: (f32, f32), // x, y
    mouse_down: bool,
}

#[derive(Default, Debug)]
struct IcedGuiInternal {
    wcmds: Vec<IcedCommand<Message>>,
    bgcolor: TurtleColor,
    resize_request: Option<(TurtleID, TurtleThread)>,
    next_id: PopupID,
    winid_to_popupid: HashMap<IcedWinId, PopupID>,
}

impl TurtleUserInterface for IcedGuiFramework {
    fn start(flags: TurtleFlags) {
        let (xsize, ysize) = (flags.size[0], flags.size[1]);

        Self::run(Settings {
            antialiasing: true,
            flags,
            window: window::Settings {
                size: Size::new(xsize, ysize),
                ..Default::default()
            },
            id: None,
            fonts: Vec::new(),
            default_font: iced::Font::default(),
            default_text_size: Pixels(16.0),
        })
        .expect("failed to start turtle");
    }
}

impl Application for IcedGuiFramework {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TurtleFlags;

    fn new(mut flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        fn new_handler(title: String) -> Handler<IcedUI, IcedGuiInternal> {
            let mut this = Handler::<IcedUI, IcedGuiInternal> {
                last_id: TurtleID::default(),
                turtle: HashMap::new(),
                title: format!(" {} ", title),
                popups: HashMap::new(),
                screen: IcedGuiInternal {
                    bgcolor: TurtleColor::from("white"),
                    next_id: PopupID::new(0),
                    ..IcedGuiInternal::default()
                },
            };
            let _turtle = this.new_turtle();
            this
        }

        let func = flags.start_func.take();

        let title = flags.title.clone();
        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let framework = Self {
            cache: Cache::default(),
            tt,
            clear_cache: true,
            handler: new_handler(title),
            winsize: (0., 0.),
            mouse_pos: (0., 0.),
            mouse_down: false,
        };

        (framework, IcedCommand::none())
    }

    fn title(&self, win_id: iced::window::Id) -> String {
        if win_id == WindowID::MAIN {
            self.handler.title.clone()
        } else {
            let popid = self.handler.screen.winid_to_popupid.get(&win_id).unwrap();
            self.handler
                .popups
                .get(popid)
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
                self.tt.tick(&mut self.handler);
                if self.update_turtles() {
                    self.clear_cache = true;
                }
            }
            Message::AckError(win_id) => {
                let popid = self.handler.screen.winid_to_popupid.get(&win_id).unwrap();
                let popup = self
                    .handler
                    .popups
                    .get_mut(popid)
                    .expect("looking up popup data");
                popup.clear_error();
            }
            Message::Event(event) => {
                let turtle_event: TurtleEvent = event.into();
                match &turtle_event {
                    TurtleEvent::WindowResize(x, y) => {
                        self.winsize = (*x as f32, *y as f32);
                        if self.handler.screen.resize_request.is_none() {
                            self.tt.handle_event(None, None, &turtle_event);
                        } else {
                            let (turtle, thread) = self
                                .handler
                                .screen
                                .resize_request
                                .expect("missing resize data");
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
                let id = self.handler.screen.winid_to_popupid.get(&id).unwrap();
                let popup = self
                    .handler
                    .popups
                    .get_mut(id)
                    .expect("looking up popup data");
                popup.set_message(&msg);
            }
            Message::TextInputSubmit(win_id) => {
                let id = self.handler.screen.winid_to_popupid.get(&win_id).unwrap();
                let mut popup = self
                    .handler
                    .popups
                    .remove(id)
                    .expect("looking up popup data");
                match popup.get_response() {
                    Ok(response) => {
                        let turtle = popup.turtle();
                        let thread = popup.thread();
                        self.tt.popup_result(turtle, thread, response);
                        self.handler.screen.wcmds.push(window::close(win_id));
                    }
                    Err(message) => {
                        popup.set_error(message);
                        self.handler.popups.insert(*id, popup);
                    }
                }
            }
            Message::Cancel(winid) => {
                let id = self.handler.screen.winid_to_popupid.get(&winid).unwrap();
                let popup = self
                    .handler
                    .popups
                    .remove(id)
                    .expect("looking up popup data");
                self.tt.popup_cancelled(popup.turtle(), popup.thread());
                self.handler.screen.wcmds.push(window::close(winid));
            }
        }
        IcedCommand::batch(self.handler.screen.wcmds.drain(..).collect::<Vec<_>>())
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
            let id = self.handler.screen.winid_to_popupid.get(&win_id).unwrap();
            let popup = self.handler.popups.get(id).expect("looking up window data");
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
            event::listen().map(|event| Message::Event(Event(event))),
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
            let ic: IcedColor = self.handler.screen.bgcolor.into();
            frame.fill_rectangle(
                [0., 0.].into(),
                bounds.size(),
                Fill {
                    style: stroke::Style::Solid(ic.into()),
                    rule: Rule::NonZero,
                },
            );
            frame.translate([center.x, center.y].into());
            for turtle in self.handler.turtle.values() {
                let ui = turtle.ui.borrow();
                ui.draw(frame);
            }
        });
        vec![geometry]
    }
}

impl IcedGuiFramework {
    // returns true if the cache should be cleared
    fn update_turtles(&mut self) -> bool {
        let mut done = true;

        for (tid, turtle) in &mut self.handler.turtle {
            let (pct, prog) = self.tt.progress(*tid);
            if turtle.has_new_cmd {
                done = false;
                let mut ui = turtle.ui.borrow_mut();
                ui.convert(pct, &turtle.cmds, turtle);
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

/*
impl Handler<IcedUI, IcedGuiInternal> {
}
*/

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Event(iced::Event);

/*
impl Deref for Event {
    type Target = iced::Event;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
*/

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

        match event.0 {
            iced::Event::Keyboard(KeyReleased { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    TurtleEvent::KeyRelease(ch)
                } else {
                    TurtleEvent::Unhandled
                }
            }
            iced::Event::Keyboard(KeyPressed { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    TurtleEvent::KeyPress(ch)
                } else {
                    TurtleEvent::Unhandled
                }
            }
            iced::Event::Window(window::Id::MAIN, Resized { width, height }) =>
            {
                #[allow(clippy::cast_possible_wrap)]
                TurtleEvent::WindowResize(width as isize, height as isize)
            }
            iced::Event::Mouse(mouse_event) => convert_mouse_event(mouse_event),
            iced::Event::Touch(_) | iced::Event::Window(..) | iced::Event::Keyboard(_) => {
                TurtleEvent::Unhandled
            }
        }
    }
}

pub trait ConvertPolygon {
    fn get_path(&self) -> Path;
}

impl ConvertPolygon for PolygonPath {
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

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct IcedColor(iced::Color);

impl Deref for IcedColor {
    type Target = iced::Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IcedColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//TODO: use tryfrom instead?
impl From<TurtleColor> for IcedColor {
    fn from(value: TurtleColor) -> Self {
        (&value).into()
    }
}

impl From<&TurtleColor> for IcedColor {
    fn from(value: &TurtleColor) -> Self {
        if let TurtleColor::Color(r, g, b) = value {
            IcedColor(iced::Color {
                r: *r,
                g: *g,
                b: *b,
                a: 1.,
            })
        } else {
            todo!()
        }
    }
}

impl From<IcedColor> for TurtleColor {
    fn from(value: IcedColor) -> Self {
        TurtleColor::Color(value.r, value.g, value.b)
    }
}

impl From<&IcedColor> for iced::Color {
    fn from(value: &IcedColor) -> Self {
        value.0
    }
}

impl From<IcedColor> for iced::Color {
    fn from(value: IcedColor) -> Self {
        value.0
    }
}
