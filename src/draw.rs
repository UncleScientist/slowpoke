use std::f64::consts::PI;

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

    pub fn setheading<N: Into<f64>>(&mut self, heading: N) {
        self.do_draw(DrawCmd::SetHeading(heading.into() - 90.));
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

    pub fn teleport<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.do_draw(DrawCmd::Teleport(xpos.into(), -ypos.into()));
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

    pub fn circle<R: Into<f64>, E: Into<f64>>(&mut self, radius: R, extent: E, steps: usize) {
        let theta_d = extent.into() / (steps as f64);
        let theta_r = theta_d * (2. * PI / 360.);
        let len = 2. * radius.into() * (theta_r / 2.).sin();

        for s in 0..steps {
            if s == 0 {
                self.left(theta_d / 2.);
            } else {
                self.left(theta_d);
            }

            self.forward(len);
        }

        self.left(theta_d / 2.);
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
