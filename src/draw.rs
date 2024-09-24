mod animation;
mod appearance;
mod circle;
mod color_control;
mod dot;
mod drawing_state;
mod filling;
mod move_and_draw;
mod screen_commands;
mod settings_for_measurement;
mod special;
mod state;

use crate::{
    command::{DataCmd, ScreenCmd},
    comms::Response,
    turtle::Turtle,
    ScreenPosition, Shape,
};

impl Turtle {
    /*
     * Other commands
     */
    pub fn hatch(&mut self) -> Turtle {
        self.do_hatch()
    }

    /*
     * Drawing commands
     */

    pub fn isvisible(&self) -> bool {
        if let Response::Visibility(can_see) = self.do_data(DataCmd::Visibility) {
            can_see
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn getscreensize(&self) -> [isize; 2] {
        let response = self.do_data(DataCmd::GetScreenSize);
        if let Response::ScreenSize(size) = response {
            size
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    pub fn turtles(&self) -> Vec<Turtle> {
        let response = self.do_data(DataCmd::GetTurtles);
        if let Response::Turtles(turtles) = response {
            turtles
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    /*
     * popup requests
     */

    pub fn textinput(&self, title: &str, prompt: &str) -> Option<String> {
        match self.do_data(DataCmd::TextInput(title.into(), prompt.into())) {
            Response::TextInput(string) => Some(string),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }

    pub fn numinput(&self, title: &str, prompt: &str) -> Option<f32> {
        match self.do_data(DataCmd::NumInput(title.into(), prompt.into())) {
            Response::NumInput(num) => Some(num),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }

    pub fn getshapes(&self) -> Vec<String> {
        if let Response::ShapeList(list) = self.do_data(DataCmd::GetShapes) {
            list
        } else {
            panic!("Unable to retrieve list of shape names");
        }
    }

    pub fn register_shape<N: ToString, S: Into<Shape>>(&mut self, name: N, shape: S) {
        self.do_screen(ScreenCmd::RegisterShape(name.to_string(), shape.into()));
    }

    pub fn addshape<N: ToString>(&mut self, name: N, shape: Shape) {
        self.register_shape(name, shape);
    }
}

impl From<&Turtle> for ScreenPosition<isize> {
    fn from(other_turtle: &Turtle) -> Self {
        other_turtle.pos()
    }
}
