pub(crate) mod popup;

use crate::{generate::DrawCommand, polygon::TurtleShape, ScreenPosition};

pub(crate) mod iced_gui;

pub(crate) trait TurtleGui: Default + Sized {
    // Generate a new connection to the windowing system
    fn new_turtle(&mut self) -> usize;

    // set the current turtle shape
    fn set_shape(&mut self, turtle_id: TurtleID, shape: TurtleShape);

    // stamp the turtle's shape onto the canvas
    fn stamp(&mut self, turtle_id: TurtleID, pos: ScreenPosition<f32>, angle: f32);

    fn get_turtle_shape_name(&mut self, turtle_id: TurtleID) -> String;

    // Call this to add a drawing command to the screen. These will be drawn
    // before the "current_command" gets drawn
    fn append_command(&mut self, turtle_id: TurtleID, cmd: DrawCommand);

    // Save the drawing position for a fill command
    fn get_position(&self, turtle_id: TurtleID) -> usize;

    // backfill a polygon at a given position
    fn fill_polygon(&mut self, turtle_id: TurtleID, cmd: DrawCommand, index: usize);

    // undo last command
    fn undo(&mut self, turtle_id: TurtleID);

    // how many commands can be undone
    fn undo_count(&self, turtle_id: TurtleID) -> usize;

    // read a numeric value from the user
    fn numinput(&mut self, turtle_id: TurtleID, which: usize, title: &str, prompt: &str);

    // read a text string from the user
    fn textinput(&mut self, turtle_id: TurtleID, which: usize, title: &str, prompt: &str);
}

#[derive(Default, Clone, Copy)]
pub(crate) enum Progression {
    #[default]
    Forward,
    Reverse,
}

pub type TurtleID = usize;

impl Progression {
    pub(crate) fn is_done(&self, pct: f32) -> bool {
        match self {
            Progression::Forward if pct >= 1. => true,
            Progression::Reverse if pct <= 0. => true,
            _ => false,
        }
    }
}
