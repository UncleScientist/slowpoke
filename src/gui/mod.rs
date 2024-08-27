pub(crate) mod events;
pub(crate) mod popup;

use crate::color_names::TurtleColor;
use crate::{generate::DrawCommand, polygon::TurtleShape, ScreenPosition};

pub(crate) mod iced_gui;

use crate::turtle::types::{TurtleID, TurtleThread};

pub(crate) trait TurtleGui: Default + Sized {
    // Generate a new connection to the windowing system
    fn new_turtle(&mut self) -> TurtleID;

    // set the current turtle shape
    fn set_shape(&mut self, turtle: TurtleID, shape: TurtleShape);

    // stamp the turtle's shape onto the canvas
    fn stamp(&mut self, turtle: TurtleID, pos: ScreenPosition<f32>, angle: f32) -> usize;

    // clear a given stamp id
    fn clear_stamp(&mut self, turtle: TurtleID, stamp: usize);

    // clear the first/last quantity of stamps
    fn clear_stamps(&mut self, turtle: TurtleID, count: StampCount);

    // get the name of the current turtle's shape
    fn get_turtle_shape_name(&mut self, turtle_id: TurtleID) -> String;

    // Call this to add a drawing command to the screen. These will be drawn
    // before the "current_command" gets drawn
    fn append_command(&mut self, turtle: TurtleID, cmd: DrawCommand);

    // Save the drawing position for a fill command
    fn get_position(&self, turtle: TurtleID) -> usize;

    // backfill a polygon at a given position
    fn fill_polygon(&mut self, turtle: TurtleID, cmd: DrawCommand, index: usize);

    // start the 'undo' drawing process
    fn undo(&mut self, turtle: TurtleID);

    // remove last command and start to undo the next
    fn pop(&mut self, turtle: TurtleID) -> Option<DrawCommand>;

    // how many commands can be undone
    fn undo_count(&self, turtle: TurtleID) -> usize;

    // read a numeric value from the user
    fn numinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str);

    // read a text string from the user
    fn textinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str);

    // set the background color
    fn bgcolor(&mut self, color: TurtleColor);

    // resize the window
    fn resize(&mut self, turtle: TurtleID, thread: TurtleThread, width: isize, height: isize);

    // show or hide the turtle
    fn set_visible(&mut self, turtle: TurtleID, visible: bool);

    // get the current visibility status
    fn is_visible(&self, turtle: TurtleID) -> bool;
}

#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum Progression {
    #[default]
    Forward,
    Reverse,
}

impl Progression {
    pub(crate) fn is_done(&self, pct: f32) -> bool {
        match self {
            Progression::Forward if pct >= 1. => true,
            Progression::Reverse if pct <= 0. => true,
            _ => false,
        }
    }
}

pub(crate) enum StampCount {
    Forward(usize), // delete from the beginning of the stamp list
    Reverse(usize), // delete from the end of the stamp list
    All,
}
