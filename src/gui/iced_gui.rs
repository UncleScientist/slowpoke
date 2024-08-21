use std::collections::HashMap;

use iced::{
    event, executor, mouse,
    multi_window::Application,
    widget::{
        button,
        canvas::{self, fill::Rule, stroke, Cache, Fill, Frame, LineJoin, Path, Stroke},
        column, container, horizontal_space, row, text, text_input, vertical_space, Canvas,
        TextInput,
    },
    window::{self, Id as WindowID},
    Color, Element, Event, Length, Point, Rectangle, Renderer, Settings, Size, Subscription, Theme,
};

use iced::keyboard::{Event::KeyPressed, Key};
use iced::window::Event::Resized;

use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};

use super::events::TurtleEvent;
use crate::{
    color_names::TurtleColor,
    generate::DrawCommand,
    gui::{popup::PopupData, TurtleGui},
    polygon::TurtleShape,
    turtle::{
        types::{TurtleID, TurtleThread},
        TurtleFlags, TurtleTask,
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
            }
        }
    }

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
                DrawCommand::Line(l) => {
                    let start: Point = [l.begin.x as f32, l.begin.y as f32].into();
                    let end: Point = if last_element {
                        let endx = l.begin.x as f32 + (l.end.x - l.begin.x) as f32 * pct;
                        let endy = l.begin.y as f32 + (l.end.y - l.begin.y) as f32 * pct;
                        tpos = [endx, endy];
                        [endx, endy]
                    } else {
                        tpos = [l.end.x as f32, l.end.y as f32];
                        [l.end.x as f32, l.end.y as f32]
                    }
                    .into();
                    if cur_path.is_empty() {
                        cur_path.push((l.pen_down, start));
                    }
                    cur_path.push((l.pen_down, end));
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
                    if points[0].pen_down {
                        let (total, subpercent) = if last_element {
                            let partial = (points.len() - 1) as f32 * pct;
                            (partial.floor() as usize, (partial - partial.floor()))
                        } else {
                            (points.len() - 1, 1_f32)
                        };
                        let path = Path::new(|b| {
                            let (_, start) = points[0].get_data();

                            b.move_to(start.into());

                            let mut iter = points.windows(2).take(total + 1).peekable();
                            while let Some(p) = iter.next() {
                                let (end_angle, end) = p[1].get_data();
                                let last_segment = iter.peek().is_none();
                                tpos = end;
                                if last_element && last_segment {
                                    let (_, begin) = p[0].get_data();
                                    let endx = begin[0] + (end[0] - begin[0]) * subpercent;
                                    let endy = begin[1] + (end[1] - begin[1]) * subpercent;
                                    tpos = [endx, endy];
                                }
                                b.line_to(tpos.into());
                                trot = end_angle;
                            }
                        });

                        self.drawing
                            .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
                    }
                }
                DrawCommand::Filler => {}
                _ => panic!("{element:?} not yet implemeted"),
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
            let path = self.turtle_shape.shape.get_path();
            let angle = Angle::degrees(trot);
            let transform = Transform2D::rotation(angle).then_translate(tpos.into());
            let path = path.transform(&transform);
            self.drawing
                .push(IcedDrawCmd::Fill(path.clone(), fillcolor));
            self.drawing
                .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
        }
    }
}

type IcedCommand<T> = iced::Command<T>;

#[derive(Default)]
pub(crate) struct IcedGuiFramework {
    cache: Cache,
    bgcolor: TurtleColor,
    tt: TurtleTask,
    gui: IcedGuiInternal,
    clear_cache: bool,
}

#[derive(Default)]
struct IcedGuiInternal {
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
    popups: HashMap<WindowID, PopupData>,
    wcmds: Vec<IcedCommand<Message>>,
    newbgcolor: Option<TurtleColor>,
}

impl TurtleGui for IcedGuiInternal {
    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(id, IndividualTurtle::default());
        id
    }

    fn set_shape(&mut self, turtle_id: TurtleID, shape: TurtleShape) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .turtle_shape = shape;
    }

    fn stamp(&mut self, turtle_id: TurtleID, pos: ScreenPosition<f32>, angle: f32) {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.cmds.push(DrawCommand::DrawPolyAt(
            turtle.turtle_shape.shape.clone(),
            pos,
            angle,
        ));
    }

    fn get_turtle_shape_name(&mut self, turtle_id: TurtleID) -> String {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.turtle_shape.name.clone()
    }

    fn append_command(&mut self, turtle_id: TurtleID, cmd: DrawCommand) {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.cmds.push(cmd);
        turtle.has_new_cmd = true;
    }

    fn get_position(&self, turtle_id: TurtleID) -> usize {
        self.turtle[&turtle_id].cmds.len()
    }

    fn fill_polygon(&mut self, turtle_id: TurtleID, cmd: DrawCommand, index: usize) {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.has_new_cmd = true;
        turtle.cmds[index] = cmd;
    }

    fn undo_count(&self, turtle_id: TurtleID) -> usize {
        self.turtle
            .get(&turtle_id)
            .expect("missing turtle")
            .cmds
            .len()
    }

    fn undo(&mut self, turtle_id: TurtleID) {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.cmds.pop();
        turtle.has_new_cmd = true;
    }

    fn numinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        self.generate_popup(PopupData::num_input(title, prompt, turtle, thread));
    }

    fn textinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        self.generate_popup(PopupData::text_input(title, prompt, turtle, thread));
    }

    fn bgcolor(&mut self, color: TurtleColor) {
        self.newbgcolor = Some(color);
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
            bgcolor: TurtleColor::from("white"),
            tt,
            clear_cache: true,
            gui: IcedGuiInternal::new(WindowID::MAIN, PopupData::mainwin(&title)),
        };

        (framework, IcedCommand::none())
    }

    fn title(&self, win_id: iced::window::Id) -> String {
        self.gui
            .popups
            .get(&win_id)
            .expect("lookup popup data")
            .title()
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
                if !matches!(turtle_event, TurtleEvent::Unhandled) {
                    self.tt.handle_event(turtle_event);
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
                    style: stroke::Style::Solid((&self.bgcolor).into()),
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
        if let Some(color) = self.gui.newbgcolor.take() {
            self.bgcolor = color;
        }

        let mut done = true;

        for (tid, turtle) in self.gui.turtle.iter_mut() {
            let (pct, prog) = self.tt.progress(*tid);
            if turtle.has_new_cmd {
                turtle.convert(pct);
                if prog.is_done(pct) {
                    turtle.has_new_cmd = false;
                } else {
                    done = false;
                }
            }
        }

        !done
    }
}

impl IcedGuiInternal {
    fn new(window_id: WindowID, popup_data: PopupData) -> Self {
        let mut this = Self {
            popups: HashMap::from([(window_id, popup_data)]),
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
    fn from(value: Event) -> Self {
        match value {
            Event::Keyboard(KeyPressed { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    TurtleEvent::KeyPress(ch)
                } else {
                    TurtleEvent::Unhandled
                }
            }
            Event::Window(window::Id::MAIN, Resized { width, height }) => {
                TurtleEvent::WindowResize(width, height)
            }
            Event::Mouse(_) => TurtleEvent::Unhandled,
            Event::Touch(_) => TurtleEvent::Unhandled,
            _ => TurtleEvent::Unhandled,
        }
    }
}
