use iced::{
    executor, multi_window::Application, widget::canvas::Path, window::Id as WindowID, Color,
    Event, Theme,
};

use crate::{generate::DrawCommand, gui::TurtleGui};

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
pub(crate) struct IcedGui {
    curcmd: Option<DrawCommand>,
    cmds: Vec<DrawCommand>,
}

impl TurtleGui for IcedGui {
    fn new_connection() -> Self {
        // need to distinguish turtles uniquely?
        Self::default()
    }

    fn current_command(&mut self, cmd: DrawCommand) {
        self.curcmd = Some(cmd);
    }

    fn append_command(&mut self, cmd: DrawCommand) {
        self.cmds.push(cmd);
    }

    fn get_position(&self) -> usize {
        self.cmds.len()
    }

    fn fill_polygon(&mut self, cmd: DrawCommand, index: usize) {
        self.cmds[index] = cmd;
    }

    fn undo(&mut self) {
        self.cmds.pop();
    }
}

impl Application for IcedGui {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = u32;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        todo!()
    }

    fn title(&self, window: iced::window::Id) -> String {
        todo!()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        todo!()
    }

    fn view(
        &self,
        window: iced::window::Id,
    ) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        todo!()
    }
}
