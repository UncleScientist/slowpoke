use std::{cell::Cell, collections::HashMap};

use iced::{
    event, executor, mouse,
    multi_window::Application,
    widget::{
        button,
        canvas::{self, fill::Rule, stroke, Cache, Fill, Frame, Path},
        column, container, horizontal_space, row, text, text_input, vertical_space, Canvas,
        TextInput,
    },
    window::{self, Id as WindowID},
    Color, Element, Event, Length, Rectangle, Renderer, Settings, Size, Subscription, Theme,
};

use crate::{
    color_names::TurtleColor,
    gui::popup::PopupData,
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
    curcmd: Option<DrawCommand>,
    cmds: Vec<DrawCommand>,
    drawing: Vec<IcedDrawCmd>,
}

impl IndividualTurtle {
    fn draw(&self, _frame: &mut Frame) {}
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

        println!("created turtle, id = {id}");
        self.turtle.insert(id, IndividualTurtle::default());
        id
    }

    fn current_command(&mut self, turtle_id: usize, cmd: DrawCommand) {
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .curcmd = Some(cmd);
    }

    fn append_command(&mut self, turtle_id: usize, cmd: DrawCommand) {
        println!("turtle {turtle_id}, cmd {cmd:?}");
        self.turtle
            .get_mut(&turtle_id)
            .expect("missing turtle")
            .cmds
            .push(cmd);
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
}

impl Application for IcedGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = TurtleFlags; // unused for now

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
                let popup = self.popups.get_mut(&id).expect("looking up popup data");
                match popup.get_response() {
                    Ok(response) => {
                        let tid = popup.id();
                        let index = popup.which();
                        // TODO: let _ = self.data[index].data.responder[&tid].send(response);
                        self.wcmds.push(window::close(id));
                    }
                    Err(message) => {
                        popup.set_error(message);
                    }
                }
            }
            Message::Cancel(id) => {
                let popup = self.popups.get(&id).expect("looking up popup data");
                // TODO: let _ = self.data[popup.which()].data.responder[&popup.id()].send(Response::Cancel);
                self.wcmds.push(window::close(id));
            }
        }
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
}
