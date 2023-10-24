use graphics::types::Vec2d;

use crate::{command::DataCmd, command::DrawCmd, command::ScreenCmd, turtle::Turtle, Response};

impl Turtle {
    /*
     * Screen commands
     */
    pub fn bgcolor<R: Into<f64>, G: Into<f64>, B: Into<f64>>(&mut self, r: R, g: G, b: B) {
        let (r, g, b): (f64, f64, f64) = (r.into(), g.into(), b.into());
        self.do_screen(ScreenCmd::Background(r as f32, g as f32, b as f32));
    }

    pub fn clearscreen(&mut self) {
        self.do_screen(ScreenCmd::ClearScreen);
    }

    /*
     * Drawing commands
     */
    pub fn pencolor<R: Into<f64>, G: Into<f64>, B: Into<f64>>(&mut self, r: R, g: G, b: B) {
        let (r, g, b): (f64, f64, f64) = (r.into(), g.into(), b.into());
        self.do_draw(DrawCmd::PenColor(r as f32, g as f32, b as f32));
    }

    pub fn penwidth<N: Into<f64>>(&mut self, width: N) {
        self.do_draw(DrawCmd::PenWidth(width.into()));
    }

    pub fn forward<N: Into<f64>>(&mut self, distance: N) {
        self.do_draw(DrawCmd::Forward(distance.into()));
    }

    pub fn backward<N: Into<f64>>(&mut self, distance: N) {
        self.do_draw(DrawCmd::Forward(-distance.into()));
    }

    pub fn right<N: Into<f64>>(&mut self, rotation: N) {
        self.do_draw(DrawCmd::Right(rotation.into()));
    }

    pub fn left<N: Into<f64>>(&mut self, rotation: N) {
        self.do_draw(DrawCmd::Left(rotation.into()));
    }

    pub fn penup(&mut self) {
        self.do_draw(DrawCmd::PenUp);
    }

    pub fn pendown(&mut self) {
        self.do_draw(DrawCmd::PenDown);
    }

    pub fn goto<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.do_draw(DrawCmd::GoTo(xpos.into(), -ypos.into()));
    }

    pub fn setx<N: Into<f64>>(&mut self, xpos: N) {
        self.do_draw(DrawCmd::SetX(xpos.into()));
    }

    pub fn sety<N: Into<f64>>(&mut self, ypos: N) {
        self.do_draw(DrawCmd::SetY(-ypos.into()));
    }

    pub fn home(&mut self) {
        self.goto(0., 0.);
    }

    /*
     * Info requests
     */
    pub fn pos(&mut self) -> Vec2d<isize> {
        if let Response::Position(pos) = self.do_data(DataCmd::Position) {
            pos
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn heading(&mut self) -> f64 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Heading) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }
}
