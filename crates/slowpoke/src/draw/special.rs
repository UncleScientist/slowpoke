use crate::{
    command::{DataCmd, DrawRequest, InstantaneousDrawCmd},
    comms::Response,
    turtle::TurtleUserInterface,
    Turtle,
};

impl Turtle {
    pub fn begin_poly(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::BeginPoly,
        ));
    }

    pub fn end_poly(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::EndPoly,
        ));
    }

    /// # Panics
    pub fn get_poly(&self) -> Vec<[f32; 2]> {
        if let Response::Polygon(polygon) = self.do_data(DataCmd::GetPoly) {
            polygon
        } else {
            panic!("invalid response from turtle");
        }
    }

    /// # Panics
    pub fn undobufferentries(&self) -> usize {
        if let Response::Count(count) = self.do_data(DataCmd::UndoBufferEntries) {
            count
        } else {
            panic!("invalid response from turtle");
        }
    }
}
