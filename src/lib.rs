use command::{Command, DrawCmd};
use graphics::types::Vec2d;
pub use turtle::{Turtle, TurtleArgs};

mod command;
mod draw;
mod input;
mod turtle;

pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub enum Response {
    Done,
    Heading(f64),
    Position(Vec2d<isize>),
}

pub struct Request {
    turtle_id: u64,
    cmd: Command,
}
