use std::sync::mpsc::{Receiver, Sender};

use crate::{Command, Request, Response};

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

    pub(crate) fn do_command(&mut self, cmd: Command) {
        self.issue_command
            .send(Request {
                turtle_id: self.turtle_id,
                cmd,
            })
            .expect("graphics window no longer exists");
        let _ = self.command_complete.recv().expect("main window died!");
    }
}
