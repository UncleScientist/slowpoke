use crate::{Command, Turtle};

impl Turtle {
    pub fn forward(&mut self, distance: f64) {
        self.issue_command
            .send(Command::Forward(distance))
            .expect("graphics window no longer exists");
        self.command_complete.recv().expect("main window died!");
    }

    pub fn right(&mut self, rotation: f64) {
        self.issue_command
            .send(Command::Right(rotation))
            .expect("graphics window no longer exists");
        self.command_complete.recv().expect("main window died!");
    }

    pub fn left(&mut self, rotation: f64) {
        self.issue_command
            .send(Command::Left(rotation))
            .expect("graphics window no longer exists");
        self.command_complete.recv().expect("main window died!");
    }
}
