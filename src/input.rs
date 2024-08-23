use crate::{command::InputCmd, turtle::Turtle};

impl Turtle {
    pub fn onkey(&self, func: fn(&mut Turtle, char), key: char) {
        self.do_input(InputCmd::OnKeyPress(func, key));
    }

    pub fn onclick(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::OnMousePress(func));
    }

    pub fn onrelease(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::OnMouseRelease(func));
    }

    pub fn ondrag(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::OnMouseDrag(func));
    }
}
