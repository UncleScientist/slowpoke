use std::f64::consts::PI;

use graphics::types::Vec2d;

use crate::{
    color_names::TurtleColor,
    command::{DataCmd, DrawCmd, ScreenCmd},
    turtle::Turtle,
    Response, StampID,
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

    pub fn clearstamp(&mut self, id: StampID) {
        self.do_screen(ScreenCmd::ClearStamp(id));
    }

    /// Clear a range of stamps. If `which` is 0, clear all stamps; if `which` is < 0, clear
    /// the last `-which` stamps, and if which is > 0, clear the first `which` stamps.
    ///
    pub fn clearstamps(&mut self, which: isize) {
        self.do_screen(ScreenCmd::ClearStamps(which));
    }

    /*
     * Drawing commands
     */
    pub fn pencolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawCmd::PenColor(color.into()));
    }

    pub fn fillcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawCmd::FillColor(color.into()));
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

    pub fn begin_fill(&mut self) {
        self.do_screen(ScreenCmd::BeginFill);
    }

    pub fn end_fill(&mut self) {
        self.do_screen(ScreenCmd::EndFill);
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

    pub fn dot<C: Into<TurtleColor>>(&mut self, width: Option<f64>, color: C) {
        self.do_draw(DrawCmd::Dot(width, color.into()));
    }

    pub fn stamp(&mut self) -> StampID {
        if let Response::StampID(id) = self.do_data(DataCmd::Stamp) {
            id
        } else {
            panic!("invalid response from turtle");
        }
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
