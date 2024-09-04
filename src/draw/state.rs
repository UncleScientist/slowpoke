use crate::{command::DataCmd, comms::Response, ScreenPosition, Turtle};

/*
 * Tell Turtle's State
 */
impl Turtle {
    pub fn position(&self) -> ScreenPosition<isize> {
        if let Response::Position(pos) = self.do_data(DataCmd::Position) {
            [pos.x, -pos.y].into()
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn pos(&self) -> ScreenPosition<isize> {
        self.position()
    }

    pub fn towards<X: Into<f64>, Y: Into<f64>>(&self, xpos: X, ypos: Y) -> f32 {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        if let Response::Heading(angle) = self.do_data(DataCmd::Towards(x, y)) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn xcor(&self) -> isize {
        self.position().x
    }

    pub fn ycor(&self) -> isize {
        self.position().y
    }

    pub fn heading(&self) -> f32 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Heading) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn distance<D: Into<ScreenPosition<isize>>>(&self, other: D) -> f64 {
        let self_pos = self.pos();
        let other_pos: ScreenPosition<isize> = other.into();

        let dx = (other_pos.x - self_pos.x) as f64;
        let dy = (other_pos.y - self_pos.y) as f64;

        (dx * dx + dy * dy).sqrt()
    }
}
