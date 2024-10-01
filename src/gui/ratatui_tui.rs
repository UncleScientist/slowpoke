use crate::turtle::TurtleFlags;

use super::TurtleGui;

#[derive(Default)]
pub struct RatatuiFramework;

impl RatatuiFramework {
    pub(crate) fn start(flags: TurtleFlags) {
        todo!()
    }
}

impl TurtleGui for RatatuiFramework {
    fn new_turtle(&mut self) -> crate::turtle::types::TurtleID {
        todo!()
    }

    fn shut_down(&mut self) {
        todo!();
    }

    fn clear_turtle(&mut self, turtle: crate::turtle::types::TurtleID) {
        todo!()
    }

    fn set_shape(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        shape: crate::polygon::TurtleShape,
    ) {
        todo!()
    }

    fn stamp(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        pos: crate::ScreenPosition<f32>,
        angle: f32,
    ) -> usize {
        todo!()
    }

    fn clear_stamp(&mut self, turtle: crate::turtle::types::TurtleID, stamp: usize) {
        todo!()
    }

    fn clear_stamps(&mut self, turtle: crate::turtle::types::TurtleID, count: super::StampCount) {
        todo!()
    }

    fn get_turtle_shape_name(&mut self, turtle_id: crate::turtle::types::TurtleID) -> String {
        todo!()
    }

    fn append_command(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        cmd: crate::generate::DrawCommand,
    ) {
        todo!()
    }

    fn get_position(&self, turtle: crate::turtle::types::TurtleID) -> usize {
        todo!()
    }

    fn fill_polygon(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        cmd: crate::generate::DrawCommand,
        index: usize,
    ) {
        todo!()
    }

    fn undo(&mut self, turtle: crate::turtle::types::TurtleID) {
        todo!()
    }

    fn pop(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
    ) -> Option<crate::generate::DrawCommand> {
        todo!()
    }

    fn undo_count(&self, turtle: crate::turtle::types::TurtleID) -> usize {
        todo!()
    }

    fn numinput(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        todo!()
    }

    fn textinput(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        todo!()
    }

    fn bgcolor(&mut self, color: crate::color_names::TurtleColor) {
        todo!()
    }

    fn resize(
        &mut self,
        turtle: crate::turtle::types::TurtleID,
        thread: crate::turtle::types::TurtleThread,
        width: isize,
        height: isize,
    ) {
        todo!()
    }

    fn set_visible(&mut self, turtle: crate::turtle::types::TurtleID, visible: bool) {
        todo!()
    }

    fn is_visible(&self, turtle: crate::turtle::types::TurtleID) -> bool {
        todo!()
    }

    fn clearscreen(&mut self) {
        todo!()
    }

    fn set_title(&mut self, title: String) {
        todo!()
    }
}
