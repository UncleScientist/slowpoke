mod circle;
mod color_control;
mod dot;
mod drawing_state;
mod filling;
mod move_and_draw;
mod settings_for_measurement;
mod state;

use crate::{
    color_names::TurtleColor,
    command::{DataCmd, DrawRequest, InstantaneousDrawCmd, MotionCmd, ScreenCmd, TimedDrawCmd},
    comms::Response,
    polygon::TurtleShapeName,
    turtle::Turtle,
    ScreenPosition, Shape,
};

impl Turtle {
    /*
     * Screen commands
     */
    pub fn bgcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_screen(ScreenCmd::Background(color.into()));
    }

    pub fn clearscreen(&mut self) {
        self.do_screen(ScreenCmd::ClearScreen);
    }

    pub fn showturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(true));
    }

    pub fn hideturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(false));
    }

    pub fn screensize<S: Into<[isize; 2]>>(&mut self, s: S) {
        self.do_screen(ScreenCmd::SetSize(s.into()));
    }

    /*
     * Other commands
     */
    pub fn hatch(&mut self) -> Turtle {
        self.do_hatch()
    }

    /*
     * Drawing commands
     */
    pub fn begin_poly(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::BeginPoly,
        ));
    }

    pub fn end_poly(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::EndPoly,
        ));
    }

    pub fn penwidth<N: Into<f64>>(&mut self, width: N) {
        let width = width.into() as f32;
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenWidth(width),
        ));
    }

    pub fn tracer(&mut self, trace: bool) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::Tracer(trace),
        ));
    }

    pub fn teleport<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Teleport(x, -y),
        )));
    }

    pub fn undobufferentries(&self) -> usize {
        if let Response::Count(count) = self.do_data(DataCmd::UndoBufferEntries) {
            count
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn shape<S: Into<TurtleShapeName>>(&self, shape: S) -> String {
        let response = self.do_data(DataCmd::TurtleShape(shape.into()));
        if let Response::Name(shape) = response {
            shape
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    pub fn get_poly(&self) -> Vec<[f32; 2]> {
        if let Response::Polygon(polygon) = self.do_data(DataCmd::GetPoly) {
            polygon
        } else {
            panic!("invalid response from turtle");
        }
    }

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
