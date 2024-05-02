use command::Command;
use graphics::types::Vec2d;
pub use polygon::TurtleShapeName;
pub use turtle::{Turtle, TurtleArgs};

pub mod color_names;
mod command;
mod draw;
mod generate;
mod input;
mod polygon;
pub mod speed;
mod turtle;

pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub type StampID = usize;

#[derive(Debug)]
pub enum Response {
    Done,
    Heading(f64),
    Position(Vec2d<isize>),
    StampID(StampID),
    Turtle(Turtle),
    Count(usize),
    Name(String),
    Polygon(Vec<[f32; 2]>),
}

pub struct Request {
    turtle_id: u64,
    cmd: Command,
}
