use graphics::types::Vec2d;
use piston::Size;

use crate::{
    color_names::TurtleColor,
    command::{
        DataCmd, DrawRequest, InstantaneousDrawCmd, MotionCmd, RotateCmd, ScreenCmd, TimedDrawCmd,
    },
    polygon::TurtleShapeName,
    speed::TurtleSpeed,
    turtle::Turtle,
    Response, StampID,
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

    pub fn begin_poly(&mut self) {
        self.do_screen(ScreenCmd::BeginPoly);
    }

    pub fn end_poly(&mut self) {
        self.do_screen(ScreenCmd::EndPoly);
    }

    pub fn showturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(true));
    }

    pub fn hideturtle(&mut self) {
        self.do_screen(ScreenCmd::ShowTurtle(false));
    }

    pub fn screensize<S: Into<Size>>(&mut self, s: S) {
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

    pub fn fillcolor<C: Into<TurtleColor>>(&mut self, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::FillColor(color.into()),
        ));
    }

    pub fn penwidth<N: Into<f64>>(&mut self, width: N) {
        self.do_draw(DrawRequest::InstantaneousDraw(
            InstantaneousDrawCmd::PenWidth(width.into()),
        ));
    }

    pub fn forward<N: Into<f64>>(&mut self, distance: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(distance.into()),
        )));
    }

    pub fn backward<N: Into<f64>>(&mut self, distance: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Forward(-distance.into()),
        )));
    }

    pub fn right<N: Into<f64>>(&mut self, rotation: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Right(rotation.into()),
        )));
    }

    pub fn left<N: Into<f64>>(&mut self, rotation: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::Left(rotation.into()),
        )));
    }

    pub fn setheading<N: Into<f64>>(&mut self, heading: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Rotate(
            RotateCmd::SetHeading(heading.into() - 90.),
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

    pub fn goto<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::GoTo(xpos.into(), -ypos.into()),
        )));
    }

    pub fn teleport<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::Teleport(xpos.into(), -ypos.into()),
        )));
    }

    pub fn setx<N: Into<f64>>(&mut self, xpos: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::SetX(xpos.into()),
        )));
    }

    pub fn sety<N: Into<f64>>(&mut self, ypos: N) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Motion(
            MotionCmd::SetY(-ypos.into()),
        )));
    }

    pub fn begin_fill(&mut self) {
        self.do_screen(ScreenCmd::BeginFill);
    }

    pub fn end_fill(&mut self) {
        self.do_screen(ScreenCmd::EndFill);
    }

    pub fn home(&mut self) {
        self.goto(0., 0.);
    }

    pub fn circle<R: Into<f64>, E: Into<f64>>(&mut self, radius: R, extent: E, steps: usize) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Circle(
            radius.into(),
            extent.into(),
            steps,
        )));
    }

    pub fn dot<C: Into<TurtleColor>>(&mut self, width: Option<f64>, color: C) {
        self.do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Dot(
            width,
            color.into(),
        )));
    }

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
    pub fn pos(&mut self) -> Vec2d<isize> {
        if let Response::Position(pos) = self.do_data(DataCmd::Position) {
            pos
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn heading(&mut self) -> f64 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Heading) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn towards<X: Into<f64>, Y: Into<f64>>(&mut self, xpos: X, ypos: Y) -> f64 {
        if let Response::Heading(angle) = self.do_data(DataCmd::Towards(xpos.into(), ypos.into())) {
            angle
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn undobufferentries(&mut self) -> usize {
        if let Response::Count(count) = self.do_data(DataCmd::UndoBufferEntries) {
            count
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn shape<S: Into<TurtleShapeName>>(&mut self, shape: S) -> String {
        if let Response::Name(shape) = self.do_data(DataCmd::TurtleShape(shape.into())) {
            shape
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn get_poly(&mut self) -> Vec<[f32; 2]> {
        if let Response::Polygon(polygon) = self.do_data(DataCmd::GetPoly) {
            polygon
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn isvisible(&mut self) -> bool {
        if let Response::Visibility(can_see) = self.do_data(DataCmd::Visibility) {
            can_see
        } else {
            panic!("invalid response from turtle");
        }
    }

    pub fn getscreensize(&mut self) -> Size {
        if let Response::ScreenSize(size) = self.do_data(DataCmd::GetScreenSize) {
            size
        } else {
            panic!("invalid response from turtle");
        }
    }
}
