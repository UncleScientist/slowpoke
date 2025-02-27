use std::time::Duration;

use crate::{
    command::InputCmd,
    turtle::{Turtle, TurtleUserInterface},
};

impl Turtle {
    pub fn onkey(&self, func: fn(&mut Turtle, char), key: char) {
        self.onkeyrelease(func, key);
    }

    pub fn onkeyrelease(&self, func: fn(&mut Turtle, char), key: char) {
        self.do_input(InputCmd::KeyRelease(func, key));
    }

    pub fn onkeypress(&self, func: fn(&mut Turtle, char), key: char) {
        self.do_input(InputCmd::KeyPress(func, key));
    }

    pub fn onclick(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::MousePress(func));
    }

    pub fn onrelease(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::MouseRelease(func));
    }

    pub fn ondrag(&self, func: fn(&mut Turtle, f32, f32)) {
        self.do_input(InputCmd::MouseDrag(func));
    }

    pub fn ontimer(&self, func: fn(&mut Turtle, Duration), duration: u64) {
        self.do_input(InputCmd::Timer(func, Duration::from_millis(duration)));
    }
}
