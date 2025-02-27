#![allow(clippy::cast_possible_truncation)]
use crate::{
    command::DataCmd, comms::Response, turtle::TurtleUserInterface, ScreenPosition, Turtle,
};

/*
 * Tell Turtle's State
 */
impl Turtle {
    /// # Panics
    /// Panics when there's a library bug
    pub fn position(&self) -> ScreenPosition<i32> {
        if let Response::Position(pos) = self.do_data(DataCmd::Position) {
            [pos.x, -pos.y].into()
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn pos(&self) -> ScreenPosition<i32> {
        self.position()
    }

    /// # Panics
    /// Panics when there's a library bug
    pub fn towards<X: Into<f64>, Y: Into<f64>>(&self, xpos: X, ypos: Y) -> f32 {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        if let Response::Heading(angle) = self.do_data(DataCmd::Towards(x, y)) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn xcor(&self) -> i32 {
        self.position().x
    }

    pub fn ycor(&self) -> i32 {
        self.position().y
    }

    /// # Panics
    /// Panics when there's a library bug
    pub fn heading(&self) -> f32 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Heading) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn distance<D: Into<ScreenPosition<i32>>>(&self, other: D) -> f64 {
        let self_pos = self.pos();
        let other_pos: ScreenPosition<i32> = other.into();

        let dx = f64::from(other_pos.x - self_pos.x);
        let dy = f64::from(other_pos.y - self_pos.y);

        (dx * dx + dy * dy).sqrt()
    }
}
