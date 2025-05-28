#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use iced::Pixels;
use slowpoke::{
    CirclePos, DrawCommand, EventResult, Handler, IndividualTurtle, LineInfo, LineSegment,
    PolygonPath, PopupData, PopupID, SlowpokeLib, TurtleColor, TurtleDraw, TurtleEvent,
    TurtleFlags, TurtleGui, TurtleID, TurtleTask, TurtleThread, TurtleUI, TurtleUserInterface,
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
pub type Turtle = slowpoke::Turtle;
pub use slowpoke::TurtleShapeName; // TODO XXX Fix this -- we shouldn't need to do this?

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
    fn draw(&self, frame: &mut Frame, ops: &[TurtleDraw]) {
        fn segs_to_path(segments: &[LineSegment]) -> Option<Path> {
            let mut iter = segments.iter();
            let mut cur = iter.next()?;
            Some(Path::new(|b| {
                let start = Point {
                    x: cur.start.x,
                    y: cur.start.y,
                };
                b.move_to(start);
                for i in iter {
                    let end = Point {
                        x: cur.end.x,
                        y: cur.end.y,
                    };
                    b.line_to(end);
                    let next_start = Point {
                        x: i.start.x,
                        y: i.start.y,
                    };
                    if next_start != end {
                        b.move_to(next_start);
                    }
                    cur = i;
                }
                let end = Point {
                    x: cur.end.x,
                    y: cur.end.y,
                };
                b.line_to(end);
            }))
        }

        for op in ops {
            match op {
                TurtleDraw::DrawLines(color, width, segments) => {
                    let color: IcedColor = color.into();
                    if let Some(path) = segs_to_path(segments) {
                        frame.stroke(
                            &path,
                            Stroke {
                                style: stroke::Style::Solid(*color),
                                width: *width,
                                line_join: LineJoin::Round,
                                ..Stroke::default()
                            },
                        );
                    }
                }
                TurtleDraw::DrawDot(center, radius, color) => {
                    let center: Point = Point::new(center.x, center.y);
                    let circle = Path::circle(center, *radius);
                    let color: IcedColor = color.into();
                    frame.fill(
                        &circle,
                        Fill {
                            style: stroke::Style::Solid(*color),
                            rule: Rule::EvenOdd,
                        },
                    );
                }
                TurtleDraw::FillPolygon(fillcolor, pencolor, pen_width, segments) => {
                    let stroke_color: IcedColor = pencolor.into();
                    let fill_color: IcedColor = fillcolor.into();
                    if let Some(path) = segs_to_path(segments) {
                        frame.fill(
                            &path,
                            Fill {
                                style: stroke::Style::Solid(*fill_color),
                                rule: Rule::EvenOdd,
                            },
                        );
                        frame.stroke(
                            &path,
                            Stroke {
                                style: stroke::Style::Solid(*stroke_color),
                                width: *pen_width,
                                line_join: LineJoin::Round,
                                ..Stroke::default()
                            },
                        );
                    }
                }
                TurtleDraw::DrawText(start_pos, text) => {}
            }
        }
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
                ui.draw(frame, &turtle.ops);
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

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Event(iced::Event);

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
