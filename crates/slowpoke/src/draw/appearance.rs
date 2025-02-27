use crate::{
    command::{DataCmd, ScreenCmd},
    comms::Response,
    turtle::TurtleUserInterface,
    Shape, Turtle, TurtleShapeName,
};

impl Turtle {
    /// # Panics
    pub fn shape<S: Into<TurtleShapeName>>(&self, shape: S) -> String {
        let response = self.do_data(DataCmd::TurtleShape(shape.into()));
        if let Response::Name(shape) = response {
            shape
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    /// # Panics
    pub fn isvisible(&self) -> bool {
        if let Response::Visibility(can_see) = self.do_data(DataCmd::Visibility) {
            can_see
        } else {
            panic!("invalid response from turtle");
        }
    }

    /// # Panics
    pub fn getshapes(&self) -> Vec<String> {
        if let Response::ShapeList(list) = self.do_data(DataCmd::GetShapes) {
            list
        } else {
            panic!("Unable to retrieve list of shape names");
        }
    }

    pub fn register_shape<N: AsRef<str>, S: Into<Shape>>(&mut self, name: N, shape: S) {
        self.do_screen(ScreenCmd::RegisterShape(
            name.as_ref().to_string(),
            shape.into(),
        ));
    }

    pub fn addshape<N: AsRef<str>>(&mut self, name: &N, shape: Shape) {
        self.register_shape(name, shape);
    }

    // TODO: pub fn resizemode(); (auto, user, noresize)
    // TODO: pub fn shapesize() / turtlesize()
    // TODO: pub fn shearfactor();
    // TODO: pub fn settiltangle();
    // TODO: pub fn tiltangle();
    // TODO: pub fn tilt();
    // TODO: pub fn shapetransform();
}
