use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd},
    turtle::TurtleUserInterface,
    Turtle,
};

impl Turtle {
    // TODO:
    // pub fn color(&mut self) -> CurrentColors {}

    pub fn pencolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenColor(color.into()),
        ));
    }

    pub fn fillcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::FillColor(color.into()),
        ));
    }
}
