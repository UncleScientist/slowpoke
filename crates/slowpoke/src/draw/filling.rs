use crate::{
    command::{DataCmd, DrawRequest, InstantaneousDrawCmd},
    comms::Response,
    Turtle,
};

impl Turtle {
    /// # Panics
    pub fn filling(&mut self) -> bool {
        if let Response::IsFilling(state) = self.do_data(DataCmd::GetFillingState) {
            state
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn begin_fill(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::BeginFill,
        ));
    }

    pub fn end_fill(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::EndFill,
        ));
    }
}
