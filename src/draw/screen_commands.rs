use crate::{
    color_names::TurtleColor,
    command::{DataCmd, ScreenCmd},
    comms::Response,
    Turtle,
};

impl Turtle {
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

    pub fn title<S: AsRef<str>>(&mut self, s: S) {
        self.do_screen(ScreenCmd::SetTitle(s.as_ref().to_string()));
    }

    /// # Panics
    pub fn getscreensize(&self) -> [isize; 2] {
        let response = self.do_data(DataCmd::GetScreenSize);
        if let Response::ScreenSize(size) = response {
            size
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    pub fn window_height(&self) -> isize {
        self.getscreensize()[1]
    }

    pub fn window_width(&self) -> isize {
        self.getscreensize()[0]
    }

    pub fn exitonclick(&mut self) {
        self.do_screen(ScreenCmd::ExitOnClick);
    }

    pub fn bye(&mut self) {
        self.do_screen(ScreenCmd::Bye);
    }
}
