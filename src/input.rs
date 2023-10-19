use piston::Key;

use crate::{turtle::Turtle, Command};

impl Turtle {
    pub fn onkey(&mut self, func: fn(&mut Turtle, Key), key: Key) {
        self.do_command(Command::OnKeyPress(func, key));
    }
}
