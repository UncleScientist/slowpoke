use crate::{color_names::TurtleColor, command::ScreenCmd, Turtle};

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
}
