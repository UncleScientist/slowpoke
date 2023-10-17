use graphics::types::Vec2d;

use crate::{turtle::Turtle, Command};

impl Turtle {
    pub fn bgcolor(&mut self, r: f32, g: f32, b: f32) {
        self.do_command(Command::Background(r, g, b));
    }

    pub fn pencolor(&mut self, r: f32, g: f32, b: f32) {
        self.do_command(Command::PenColor(r, g, b));
    }

    pub fn penwidth(&mut self, width: f64) {
        self.do_command(Command::PenWidth(width));
    }

    pub fn forward(&mut self, distance: f64) {
        self.do_command(Command::Forward(distance));
    }

    pub fn backward(&mut self, distance: f64) {
        self.do_command(Command::Forward(-distance));
    }

    pub fn right(&mut self, rotation: f64) {
        self.do_command(Command::Right(rotation));
    }

    pub fn left(&mut self, rotation: f64) {
        self.do_command(Command::Left(rotation));
    }

    pub fn penup(&mut self) {
        self.do_command(Command::PenUp);
    }

    pub fn pendown(&mut self) {
        self.do_command(Command::PenDown);
    }

    pub fn goto(&mut self, xpos: f64, ypos: f64) {
        self.do_command(Command::GoTo(xpos, ypos));
    }

    pub fn home(&mut self) {
        self.goto(0., 0.);
    }

    pub fn pos(&self) -> Vec2d<isize> {
        // self.current_pos
        Vec2d::default()
    }

    pub fn heading(&self) -> f64 {
        // self.angle
        0.
    }

    pub fn clearscreen(&mut self) {
        self.do_command(Command::ClearScreen);
    }
}
