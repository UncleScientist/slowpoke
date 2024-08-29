use lyon_tessellation::geom::euclid::{Point2D, UnknownUnit};
pub use polygon::TurtleShapeName;
pub use turtle::{Turtle, TurtleArgs};

pub mod color_names;
mod command;
mod comms;
mod draw;
mod generate;
mod gui;
mod input;
mod polygon;
pub mod speed;
mod turtle;

pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub type ScreenCoords = UnknownUnit;
pub type ScreenPosition<T> = Point2D<T, ScreenCoords>;

pub type StampID = usize;
