use crate::{
    command::{DataCmd, DrawRequest, InstantaneousDrawCmd},
    comms::Response,
    Turtle,
};

impl Turtle {
    pub fn pendown(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenDown,
        ));
    }

    pub fn pd(&mut self) {
        self.pendown();
    }

    pub fn down(&mut self) {
        self.pendown();
    }

    pub fn penup(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::PenUp));
    }

    pub fn pu(&mut self) {
        self.penup();
    }

    pub fn up(&mut self) {
        self.penup();
    }

    // TODO
    // let attributes = turtle.pen(Some(PenAttributes {
    //    shown: Some(true),
    //    pensize: Some(4),
    //    ..Self::default()
    // }));
    //
    // let att = turtle.pen(vec![PenAttrib::Shown(true), PenAttrib::Size(4)]);
    //
    // let current = turtle.pen(PenAttributes::default());
    //
    // let attributes = turtle.pen().shown(true).pensize(4).done();

    /// # Panics
    pub fn isdown(&mut self) -> bool {
        if let Response::IsPenDown(state) = self.do_data(DataCmd::GetPenState) {
            state
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn pensize<N: Into<f64>>(&mut self, width: N) {
        #[allow(clippy::cast_possible_truncation)]
        let width = width.into() as f32;
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenWidth(width),
        ));
    }

    pub fn width<N: Into<f64>>(&mut self, width: N) {
        self.pensize(width);
    }

    pub fn clear(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Clear));
    }

    pub fn reset(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Reset));
    }

    pub fn write(&mut self, text: &str) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Text(
            text.to_string(),
        )));
    }
}
