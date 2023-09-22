use crate::{Command, Turtle};

impl Turtle {
    pub fn forward(&mut self, distance: f64) {
        self.do_command(Command::Forward(distance));
    }

    pub fn right(&mut self, rotation: f64) {
        self.do_command(Command::Right(rotation));
    }

    pub fn left(&mut self, rotation: f64) {
        self.do_command(Command::Left(rotation));
    }

    pub fn penup(&mut self) {
        self.do_command(Command::PenUp);
    }

    pub fn pendown(&mut self) {
        self.do_command(Command::PenDown);
    }

    pub fn goto(&mut self, xpos: f64, ypos: f64) {
        self.do_command(Command::GoTo(xpos, ypos));
    }

    fn do_command(&mut self, cmd: Command) {
        self.issue_command
            .send(cmd)
            .expect("graphics window no longer exists");
        self.command_complete.recv().expect("main window died!");
    }
}
