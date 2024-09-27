use std::f32::consts::PI;

use crate::{
    command::{DrawRequest, InstantaneousDrawCmd},
    Turtle,
};

impl Turtle {
    pub fn degrees<D: Into<f64>>(&mut self, fullcircle: D) {
        #[allow(clippy::cast_possible_truncation)]
        let degrees = fullcircle.into() as f32;
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::SetDegrees(degrees),
        ));
    }

    pub fn radians(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::SetDegrees(2. * PI),
        ));
    }
}
