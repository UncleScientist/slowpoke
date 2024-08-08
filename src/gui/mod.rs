use crate::generate::DrawCommand;

pub(crate) mod iced_gui;

pub(crate) trait TurtleGui: Default + Sized {
    // Generate a new connection to the windowing system
    fn new_connection() -> Self;

    // Call this to update the current location of the turtle, along with
    // whatever line is being drawn behind it (if any)
    fn current_command(&mut self, cmd: DrawCommand);

    // Call this to add a drawing command to the screen. These will be drawn
    // before the "current_command" gets drawn
    fn append_command(&mut self, cmd: DrawCommand);

    // Save the drawing position for a fill command
    fn get_position(&self) -> usize;

    // backfill a polygon at a given position
    fn fill_polygon(&mut self, cmd: DrawCommand, index: usize);

    // undo last command
    fn undo(&mut self);

    // needs functions for:
    //  - numinput
    //  - textinput
}
