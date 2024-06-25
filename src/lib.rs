use command::Command;
use iced::Size;
use lyon_tessellation::geom::euclid::{Point2D, UnknownUnit};
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

pub type ScreenCoords = UnknownUnit;
pub type ScreenPosition<T> = Point2D<T, ScreenCoords>;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Key;

/*
impl ScreenCoords {
    fn from<T: Copy>(value: [T; 2]) -> ScreenPosition<T> {
        Point2D::new(value[0] as T, value[1] as T)
    }
}
*/

pub type StampID = usize;

#[derive(Debug)]
pub enum Response {
    Done,
    Heading(f32),
    Position(ScreenPosition<isize>),
    StampID(StampID),
    Turtle(Turtle),
    Count(usize),
    Name(String),
    Polygon(Vec<[f32; 2]>),
    Visibility(bool),
    ScreenSize(Size),
}

#[derive(Debug)]
pub struct Request {
    turtle_id: u64,
    cmd: Command,
}
