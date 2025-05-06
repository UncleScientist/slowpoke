use lyon_tessellation::geom::euclid::{Point2D, UnknownUnit};
pub use polygon::{Shape, TurtleShapeName};
pub use turtle::{SlowpokeLib, Turtle};

pub mod color_names;
mod command;
mod comms;
mod draw;
mod generate;
mod gui;
mod polygon;
pub mod speed;
mod turtle;
mod user_events;

pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub type ScreenCoords = UnknownUnit;
pub type ScreenPosition<T> = Point2D<T, ScreenCoords>;

pub type StampID = usize;

// make these all top-level exports
pub use color_names::TurtleColor;
pub use generate::{CirclePos, DrawCommand, LineInfo};
pub use gui::{
    events::TurtleEvent,
    ops::{LineSegment, TurtleDraw},
    popup::PopupData,
    TurtleGui,
};
pub use polygon::{PolygonPath, ShapeComponent};
pub use turtle::handler::{Handler, IndividualTurtle, TurtleUI};
pub use turtle::task::{EventResult, TurtleTask};
pub use turtle::types::{PopupID, TurtleID, TurtleThread};
pub use turtle::{TurtleFlags, TurtleUserInterface};
