use std::f32::consts::PI;

use crate::{
    command::{DrawRequest, InstantaneousDrawCmd},
    Turtle,
};

impl Turtle {
    pub fn degrees(&mut self, fullcircle: f32) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::SetDegrees(fullcircle),
        ));
    }

    pub fn radians(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::SetDegrees(2. * PI),
        ));
    }
}
