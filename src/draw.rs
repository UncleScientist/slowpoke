use graphics::types::Vec2d;

use crate::{turtle::Turtle, DrawCmd, ScreenCmd};

impl Turtle {
    /*
     * Screen commands
     */
    pub fn bgcolor(&mut self, r: f32, g: f32, b: f32) {
        self.do_screen(ScreenCmd::Background(r, g, b));
    }

    pub fn clearscreen(&mut self) {
        self.do_screen(ScreenCmd::ClearScreen);
    }

    /*
     * Drawing commands
     */
    pub fn pencolor(&mut self, r: f32, g: f32, b: f32) {
        self.do_draw(DrawCmd::PenColor(r, g, b));
    }

    pub fn penwidth(&mut self, width: f64) {
        self.do_draw(DrawCmd::PenWidth(width));
    }

    pub fn forward(&mut self, distance: f64) {
        self.do_draw(DrawCmd::Forward(distance));
    }

    pub fn backward(&mut self, distance: f64) {
        self.do_draw(DrawCmd::Forward(-distance));
    }

    pub fn right(&mut self, rotation: f64) {
        self.do_draw(DrawCmd::Right(rotation));
    }

    pub fn left(&mut self, rotation: f64) {
        self.do_draw(DrawCmd::Left(rotation));
    }

    pub fn penup(&mut self) {
        self.do_draw(DrawCmd::PenUp);
    }

    pub fn pendown(&mut self) {
        self.do_draw(DrawCmd::PenDown);
    }

    pub fn goto(&mut self, xpos: f64, ypos: f64) {
        self.do_draw(DrawCmd::GoTo(xpos, ypos));
    }

    pub fn home(&mut self) {
        self.goto(0., 0.);
    }

    /*
     * Info requests
     */
    pub fn pos(&self) -> Vec2d<isize> {
        // self.current_pos
        Vec2d::default()
    }

    pub fn heading(&self) -> f64 {
        // self.angle
        0.
    }
}
