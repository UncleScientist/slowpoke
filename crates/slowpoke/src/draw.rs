mod animation;
mod appearance;
mod circle;
mod color_control;
mod dot;
mod drawing_state;
mod filling;
mod input_methods;
mod move_and_draw;
mod screen_commands;
mod settings_for_measurement;
mod special;
mod state;

use std::{fs::File, io::Read, path::Path};

use crate::{
    command::{DataCmd, ScreenCmd},
    comms::Response,
    turtle::Turtle,
    ScreenPosition,
};

impl Turtle {
    /*
     * Other commands
     */
    #[must_use]
    pub fn hatch(&mut self) -> Self {
        self.do_hatch()
    }

    /// # Panics
    /// Panics when it can't read the file
    pub fn bgpic<P: AsRef<Path>>(&mut self, path: P) {
        let mut file = File::open(path.as_ref()).expect("couldn't open file");
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).expect("couldn't read file");
        self.do_screen(ScreenCmd::BgPic(vec));
    }

    /// # Panics
    /// Panics when there's a library bug
    pub fn turtles(&self) -> Vec<Turtle> {
        let response = self.do_data(DataCmd::GetTurtles);
        if let Response::Turtles(turtles) = response {
            turtles
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }
}

// TODO: move to src/turtle/types.rs ??
impl From<&Turtle> for ScreenPosition<i32> {
    fn from(other_turtle: &Turtle) -> Self {
        other_turtle.pos()
    }
}
