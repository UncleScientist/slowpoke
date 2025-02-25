use crate::{
    command::{DrawRequest, InstantaneousDrawCmd},
    Turtle,
};

impl Turtle {
    pub fn tracer(&mut self, trace: bool) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::Tracer(trace),
        ));
    }

    // TODO: set delay in ms (use 0 to get current delay)
    // pub fn delay(delay: usize) -> usize {}

    // TODO: update the screen (when tracer is off)
    // pub fn update() {}
}
