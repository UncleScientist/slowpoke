use crate::{
    command::{DataCmd, DrawRequest, MotionCmd, RotateCmd, ScreenCmd, TimedDrawCmd},
    comms::Response,
    speed::Speed,
    StampID, Turtle,
};

impl Turtle {
    /*
     * Move and draw
     */

    pub fn forward<N: Copy + Into<f64>>(&mut self, distance: N) {
        #[allow(clippy::cast_possible_truncation)]
        let distance = distance.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(distance),
        )));
    }

    pub fn fd<N: Copy + Into<f64>>(&mut self, distance: N) {
        self.forward(distance);
    }

    pub fn backward<N: Into<f64>>(&mut self, distance: N) {
        #[allow(clippy::cast_possible_truncation)]
        let distance = distance.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(-distance),
        )));
    }

    pub fn bk<N: Into<f64>>(&mut self, distance: N) {
        self.backward(distance);
    }

    pub fn back<N: Into<f64>>(&mut self, distance: N) {
        self.backward(distance);
    }

    pub fn right<N: Into<f64>>(&mut self, rotation: N) {
        #[allow(clippy::cast_possible_truncation)]
        let rotation = rotation.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Right(rotation),
        )));
    }

    pub fn rt<N: Into<f64>>(&mut self, rotation: N) {
        self.right(rotation);
    }

    pub fn left<N: Into<f64>>(&mut self, rotation: N) {
        #[allow(clippy::cast_possible_truncation)]
        let rotation = rotation.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Left(rotation),
        )));
    }

    pub fn lt<N: Into<f64>>(&mut self, rotation: N) {
        self.left(rotation);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn goto<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::GoTo(x, -y),
        )));
    }

    pub fn setpos<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.goto(xpos, ypos);
    }

    pub fn setposition<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.goto(xpos, ypos);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn teleport<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Teleport(x, -y),
        )));
    }

    pub fn setx<N: Into<f64>>(&mut self, xpos: N) {
        #[allow(clippy::cast_possible_truncation)]
        let x = xpos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::SetX(x),
        )));
    }

    pub fn sety<N: Into<f32>>(&mut self, ypos: N) {
        let y = ypos.into();
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::SetY(-y),
        )));
    }

    pub fn setheading<N: Into<f64>>(&mut self, heading: N) {
        #[allow(clippy::cast_possible_truncation)]
        let heading = heading.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::SetHeading(heading - 90.),
        )));
    }

    pub fn seth<N: Into<f64>>(&mut self, heading: N) {
        self.setheading(heading);
    }

    pub fn home(&mut self) {
        self.goto(0, 0);
    }

    // this changes the turtle's state even though we're calling do_data(), so
    // we need to pass in a mutable reference
    /// # Panics
    pub fn stamp(&mut self) -> StampID {
        let response = self.do_data(DataCmd::Stamp);
        if let Response::StampID(id) = response {
            id
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    pub fn clearstamp(&mut self, id: StampID) {
        self.do_screen(ScreenCmd::ClearStamp(id));
    }

    /// Clear a range of stamps. If `which` is 0, clear all stamps; if `which` is < 0, clear
    /// the last `-which` stamps, and if which is > 0, clear the first `which` stamps.
    pub fn clearstamps(&mut self, which: isize) {
        self.do_screen(ScreenCmd::ClearStamps(which));
    }

    pub fn undo(&mut self) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Undo));
    }

    pub fn speed<S: Into<Speed>>(&mut self, speed: S) {
        self.do_screen(ScreenCmd::Speed(speed.into()));
    }
}
