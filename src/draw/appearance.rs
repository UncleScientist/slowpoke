use crate::{command::DataCmd, comms::Response, Turtle, TurtleShapeName};

impl Turtle {
    pub fn shape<S: Into<TurtleShapeName>>(&self, shape: S) -> String {
        let response = self.do_data(DataCmd::TurtleShape(shape.into()));
        if let Response::Name(shape) = response {
            shape
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    // TODO: pub fn resizemode(); (auto, user, noresize)
    // TODO: pub fn shapesize() / turtlesize()
    // TODO: pub fn shearfactor();
    // TODO: pub fn settiltangle();
    // TODO: pub fn tiltangle();
    // TODO: pub fn tilt();
    // TODO: pub fn shapetransform();
}
