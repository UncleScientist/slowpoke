use std::sync::mpsc::{Receiver, Sender};

use crate::{
    command::Command, command::DataCmd, command::InputCmd, command::ScreenCmd, DrawCmd, Request,
    Response,
};

pub struct Turtle {
    issue_command: Sender<Request>,
    command_complete: Receiver<Response>,
    turtle_id: u64,
}

impl Turtle {
    pub(crate) fn new(
        issue_command: Sender<Request>,
        command_complete: Receiver<Response>,
        turtle_id: u64,
    ) -> Self {
        Self {
            issue_command,
            command_complete,
            turtle_id,
        }
    }

    pub(crate) fn do_draw(&mut self, cmd: DrawCmd) {
        let _ = self.do_command(Command::Draw(cmd));
    }

    pub(crate) fn do_screen(&mut self, cmd: ScreenCmd) {
        let _ = self.do_command(Command::Screen(cmd));
    }

    pub(crate) fn do_input(&mut self, cmd: InputCmd) {
        let _ = self.do_command(Command::Input(cmd));
    }

    pub(crate) fn do_data(&mut self, cmd: DataCmd) -> Response {
        self.do_command(Command::Data(cmd))
    }

    fn do_command(&mut self, cmd: Command) -> Response {
        self.issue_command
            .send(Request {
                turtle_id: self.turtle_id,
                cmd,
            })
            .expect("graphics window no longer exists");
        self.command_complete.recv().expect("main window died!")
    }
}
