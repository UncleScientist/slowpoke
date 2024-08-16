use std::{cell::Cell, collections::HashMap};

use iced::{
    event, executor, mouse,
    multi_window::Application,
    widget::{
        button,
        canvas::{self, fill::Rule, stroke, Cache, Fill, Frame, Path, Stroke},
        column, container, horizontal_space, row, text, text_input, vertical_space, Canvas,
        TextInput,
    },
    window::{self, Id as WindowID},
    Color, Element, Event, Length, Point, Rectangle, Renderer, Settings, Size, Subscription, Theme,
};

use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};

use crate::{
    color_names::TurtleColor,
    gui::popup::PopupData,
    polygon::TurtleShape,
    turtle::{TurtleID, TurtleTask},
};
use crate::{generate::DrawCommand, gui::TurtleGui, turtle::TurtleFlags};

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
    _curcmd: Option<DrawCommand>,
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

    fn convert(&mut self) {
        let mut pencolor = Color::BLACK;
        let mut penwidth = 1.0;
        let mut fillcolor = Color::BLACK;
        let pct = 1.;

        let mut tpos = [0f32, 0f32];
        let mut trot = 0f32;

        self.drawing.clear();

        let mut iter = self.cmds.iter().peekable();
        let mut cur_path: Vec<Point> = Vec::new();

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            if !matches!(element, DrawCommand::Line(..))
                && !matches!(element, DrawCommand::SetHeading(..))
                && !cur_path.is_empty()
            {
                let path = Path::new(|b| {
                    b.move_to(cur_path[0]);
                    for pos in &cur_path[1..] {
                        b.line_to(*pos);
                    }
                });
                cur_path = Vec::new();
                self.drawing
                    .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
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
                    if l.pen_down {
                        if cur_path.is_empty() {
                            cur_path.push(start);
                        }
                        cur_path.push(end);
                    }
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
            let path = Path::new(|b| {
                b.move_to(cur_path[0]);
                for pos in &cur_path[1..] {
                    b.line_to(*pos);
                }
            });
            self.drawing
                .push(IcedDrawCmd::Stroke(path, pencolor, penwidth));
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
pub(crate) struct IcedGui {
    cache: Cache,
    bgcolor: TurtleColor,
    last_id: TurtleID,
    turtle: HashMap<usize, IndividualTurtle>,
    tt: Cell<TurtleTask>,
    popups: HashMap<WindowID, PopupData>,
    wcmds: Vec<IcedCommand<Message>>,
}

impl TurtleGui for IcedGui {
    fn new_turtle(&mut self) -> usize {
        let id = self.last_id;
        self.last_id += 1;

        self.turtle.insert(id, IndividualTurtle::default());
        id
    }

    fn set_shape(&mut self, turtle_id: usize, shape: TurtleShape) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .turtle_shape = shape;
    }

    fn current_command(&mut self, turtle_id: usize, cmd: DrawCommand) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            ._curcmd = Some(cmd);
    }

    fn append_command(&mut self, turtle_id: usize, cmd: DrawCommand) {
        let turtle = self.turtle.get_mut(&turtle_id).expect("missing turtle");
        turtle.cmds.push(cmd);
        turtle.has_new_cmd = true;
    }

    fn get_position(&self, turtle_id: usize) -> usize {
        self.turtle[&turtle_id].cmds.len()
    }

    fn fill_polygon(&mut self, turtle_id: usize, cmd: DrawCommand, index: usize) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .cmds[index] = cmd;
    }

    fn undo(&mut self, turtle_id: usize) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .cmds
            .pop();
    }

    fn numinput(&mut self, turtle_id: usize, which: usize, title: &str, prompt: &str) {
        self.generate_popup(PopupData::num_input(title, prompt, turtle_id, which));
    }

    fn textinput(&mut self, turtle_id: usize, which: usize, title: &str, prompt: &str) {
        self.generate_popup(PopupData::text_input(title, prompt, turtle_id, which));
    }
}

impl Application for IcedGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TurtleFlags;

    fn new(mut flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let func = flags.start_func.take();

        let title = flags.title.clone();
        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap(), 0);

        let mut gui = Self {
            tt: Cell::new(tt),
            cache: Cache::default(),
            bgcolor: TurtleColor::from("white"),
            popups: HashMap::from([(WindowID::MAIN, PopupData::mainwin(&title))]),
            ..Self::default()
        };
        let turtle_id = gui.new_turtle();
        assert_eq!(turtle_id, 0);
        (gui, IcedCommand::none())
    }

    fn title(&self, win_id: iced::window::Id) -> String {
        self.popups.get(&win_id).expect("lookup popup data").title()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Tick => {
                self.cache.clear();
                let mut tt = self.tt.take();
                tt.tick(self);
                self.tt.replace(tt);
            }
            Message::AckError(id) => {
                let popup = self.popups.get_mut(&id).expect("looking up popup data");
                popup.clear_error();
            }
            Message::Event(event) => {
                // TODO: Translate `event` into a TurtleEvent. Also, invent the TurtleEvent type
                let mut tt = self.tt.take();
                tt.handle_event(event, self);
                self.tt.replace(tt);
            }
            Message::TextInputChanged(id, msg) => {
                let popup = self.popups.get_mut(&id).expect("looking up popup data");
                popup.set_message(&msg);
            }
            Message::TextInputSubmit(id) => {
                let mut popup = self.popups.remove(&id).expect("looking up popup data");
                match popup.get_response() {
                    Ok(response) => {
                        let tid = popup.id() as usize;
                        let index = popup.which();
                        self.tt.get_mut().popup_result(tid, index, response);
                        self.wcmds.push(window::close(id));
                    }
                    Err(message) => {
                        popup.set_error(message);
                        self.popups.insert(id, popup);
                    }
                }
            }
            Message::Cancel(id) => {
                let popup = self.popups.remove(&id).expect("looking up popup data");
                self.tt.get_mut().popup_cancelled(popup.id(), popup.which());
                self.wcmds.push(window::close(id));
            }
        }
        self.convert_to_iced();
        IcedCommand::batch(self.wcmds.drain(..).collect::<Vec<_>>())
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
            let popup = self.popups.get(&win_id).expect("looking up window data");
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

impl<Message> canvas::Program<Message> for IcedGui {
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
            for turtle in self.turtle.values() {
                turtle.draw(frame);
            }
        });
        vec![geometry]
    }
}

impl IcedGui {
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

    fn convert_to_iced(&mut self) {
        for (_, turtle) in self.turtle.iter_mut() {
            if turtle.has_new_cmd {
                turtle.convert();
            }
        }
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
