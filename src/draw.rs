mod circle;
mod dot;

use crate::{
    color_names::TurtleColor,
    command::{
        DataCmd, DrawRequest, InstantaneousDrawCmd, MotionCmd, RotateCmd, ScreenCmd, TimedDrawCmd,
    },
    comms::Response,
    polygon::TurtleShapeName,
    speed::TurtleSpeed,
    turtle::Turtle,
    ScreenPosition, StampID,
};

impl Turtle {
    /*
     * Screen commands
     */
    pub fn bgcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_screen(ScreenCmd::Background(color.into()));
    }

    pub fn clearscreen(&mut self) {
        self.do_screen(ScreenCmd::ClearScreen);
    }

    pub fn clearstamp(&mut self, id: StampID) {
        self.do_screen(ScreenCmd::ClearStamp(id));
    }

    pub fn speed<S: Into<TurtleSpeed>>(&mut self, speed: S) {
        self.do_screen(ScreenCmd::Speed(speed.into()));
    }

    pub fn showturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(true));
    }

    pub fn hideturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(false));
    }

    pub fn screensize<S: Into<[isize; 2]>>(&mut self, s: S) {
        self.do_screen(ScreenCmd::SetSize(s.into()));
    }

    /// Clear a range of stamps. If `which` is 0, clear all stamps; if `which` is < 0, clear
    /// the last `-which` stamps, and if which is > 0, clear the first `which` stamps.
    ///
    pub fn clearstamps(&mut self, which: isize) {
        self.do_screen(ScreenCmd::ClearStamps(which));
    }

    /*
     * Other commands
     */
    pub fn hatch(&mut self) -> Turtle {
        self.do_hatch()
    }

    /*
     * Drawing commands
     */
    pub fn pencolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenColor(color.into()),
        ));
    }

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

    pub fn fillcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::FillColor(color.into()),
        ));
    }

    pub fn penwidth<N: Into<f64>>(&mut self, width: N) {
        let width = width.into() as f32;
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenWidth(width),
        ));
    }

    pub fn fd<N: Copy + Into<f64>>(&mut self, distance: N) {
        self.forward(distance);
    }

    pub fn forward<N: Copy + Into<f64>>(&mut self, distance: N) {
        let distance = distance.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(distance),
        )));
    }

    pub fn bk<N: Into<f64>>(&mut self, distance: N) {
        self.backward(distance);
    }

    pub fn back<N: Into<f64>>(&mut self, distance: N) {
        self.backward(distance);
    }

    pub fn backward<N: Into<f64>>(&mut self, distance: N) {
        let distance = distance.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(-distance),
        )));
    }

    pub fn rt<N: Into<f64>>(&mut self, rotation: N) {
        self.right(rotation);
    }

    pub fn right<N: Into<f64>>(&mut self, rotation: N) {
        let rotation = rotation.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Right(rotation),
        )));
    }

    pub fn lt<N: Into<f64>>(&mut self, rotation: N) {
        self.left(rotation);
    }

    pub fn left<N: Into<f64>>(&mut self, rotation: N) {
        let rotation = rotation.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Left(rotation),
        )));
    }

    pub fn seth<N: Into<f64>>(&mut self, heading: N) {
        self.setheading(heading);
    }

    pub fn setheading<N: Into<f64>>(&mut self, heading: N) {
        let heading = heading.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::SetHeading(heading - 90.),
        )));
    }

    pub fn penup(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::PenUp));
    }

    pub fn pendown(&mut self) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenDown,
        ));
    }

    pub fn tracer(&mut self, trace: bool) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::Tracer(trace),
        ));
    }

    pub fn setpos<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.goto(xpos, ypos);
    }

    pub fn setposition<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.goto(xpos, ypos);
    }

    pub fn goto<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::GoTo(x, -y),
        )));
    }

    pub fn teleport<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Teleport(x, -y),
        )));
    }

    pub fn setx<N: Into<f64>>(&mut self, xpos: N) {
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

    pub fn home(&mut self) {
        self.goto(0., 0.);
    }

    // TODO: this changes the turtle's state even though we're calling do_data()
    pub fn stamp(&mut self) -> StampID {
        let response = self.do_data(DataCmd::Stamp);
        if let Response::StampID(id) = response {
            id
        } else {
            panic!("invalid response from turtle: {response:?}");
        }
    }

    pub fn undo(&mut self) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Undo));
    }

    /*
     * Info requests
     */
    pub fn pos(&self) -> ScreenPosition<isize> {
        if let Response::Position(pos) = self.do_data(DataCmd::Position) {
            [pos.x, -pos.y].into()
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn position(&self) -> ScreenPosition<isize> {
        self.pos()
    }

    pub fn xcor(&self) -> isize {
        self.pos().x
    }

    pub fn ycor(&self) -> isize {
        self.pos().y
    }

    pub fn distance<D: Into<ScreenPosition<isize>>>(&self, other: D) -> f64 {
        let self_pos = self.pos();
        let other_pos: ScreenPosition<isize> = other.into();

        let dx = (other_pos.x - self_pos.x) as f64;
        let dy = (other_pos.y - self_pos.y) as f64;

        (dx * dx + dy * dy).sqrt()
    }

    pub fn heading(&self) -> f32 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Heading) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn towards<X: Into<f64>, Y: Into<f64>>(&self, xpos: X, ypos: Y) -> f32 {
        let x = xpos.into() as f32;
        let y = ypos.into() as f32;
        if let Response::Heading(angle) = self.do_data(DataCmd::Towards(x, y)) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn undobufferentries(&self) -> usize {
        if let Response::Count(count) = self.do_data(DataCmd::UndoBufferEntries) {
            count
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn shape<S: Into<TurtleShapeName>>(&self, shape: S) -> String {
        if let Response::Name(shape) = self.do_data(DataCmd::TurtleShape(shape.into())) {
            shape
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn get_poly(&self) -> Vec<[f32; 2]> {
        if let Response::Polygon(polygon) = self.do_data(DataCmd::GetPoly) {
            polygon
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn isvisible(&self) -> bool {
        if let Response::Visibility(can_see) = self.do_data(DataCmd::Visibility) {
            can_see
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn getscreensize(&self) -> [isize; 2] {
        let response = self.do_data(DataCmd::GetScreenSize);
        if let Response::ScreenSize(size) = response {
            size
        } else {
            panic!("{}", format!("invalid response from turtle: {response:?}"));
        }
    }

    /*
     * popup requests
     */

    pub fn textinput(&self, title: &str, prompt: &str) -> Option<String> {
        match self.do_data(DataCmd::TextInput(title.into(), prompt.into())) {
            Response::TextInput(string) => Some(string),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }

    pub fn numinput(&self, title: &str, prompt: &str) -> Option<f32> {
        match self.do_data(DataCmd::NumInput(title.into(), prompt.into())) {
            Response::NumInput(num) => Some(num),
            Response::Cancel => None,
            bad_response => panic!("invalid response '{bad_response:?}' from turtle"),
        }
    }
}

impl From<&Turtle> for ScreenPosition<isize> {
    fn from(other_turtle: &Turtle) -> Self {
        other_turtle.pos()
    }
}
