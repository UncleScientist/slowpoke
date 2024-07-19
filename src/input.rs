use crate::{command::InputCmd, turtle::Turtle};

impl Turtle {
    pub fn onkey(&self, func: fn(&mut Turtle, char), key: char) {
        self.do_input(InputCmd::OnKeyPress(func, key));
    }
}
