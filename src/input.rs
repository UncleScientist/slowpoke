use piston::Key;

use crate::{turtle::Turtle, InputCmd};

impl Turtle {
    pub fn onkey(&mut self, func: fn(&mut Turtle, Key), key: Key) {
        self.do_input(InputCmd::OnKeyPress(func, key));
    }
}
