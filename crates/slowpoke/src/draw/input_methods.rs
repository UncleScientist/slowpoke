use crate::{command::DataCmd, comms::Response, turtle::TurtleUserInterface, Turtle};

impl Turtle {
    /// # Panics
    pub fn textinput(&self, title: &str, prompt: &str) -> Option<String> {
        match self.do_data(DataCmd::TextInput(title.into(), prompt.into())) {
            Response::TextInput(string) => Some(string),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }

    /// # Panics
    pub fn numinput(&self, title: &str, prompt: &str) -> Option<f32> {
        match self.do_data(DataCmd::NumInput(title.into(), prompt.into())) {
            Response::NumInput(num) => Some(num),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }
}
