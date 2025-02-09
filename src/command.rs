use std::time::Duration;

use crate::{color_names::TurtleColor, polygon::TurtleShapeName, speed::Speed, Shape, Turtle};

//
// A DrawRequest is something that the turtle thread asks us to put on the screen.
// This is different to a DrawCommand, which is responsible for containing the
// rendering information.
#[derive(Clone, Debug)]
pub enum DrawRequest {
    TimedDraw(TimedDrawCmd),
    InstantaneousDraw(InstantaneousDrawCmd),
}

// commands that draw but don't return anything
#[derive(Clone, Debug)]
pub enum TimedDrawCmd {
    Motion(MotionCmd),
    Rotate(RotateCmd),
    Circle(f32, f32, usize),
    Undo,
}

#[derive(Clone, Debug)]
pub enum MotionCmd {
    Forward(f32),
    GoTo(f32, f32),
    Teleport(f32, f32),
    SetX(f32),
    SetY(f32),
}

#[derive(Clone, Debug)]
pub enum RotateCmd {
    Right(f32),
    Left(f32),
    SetHeading(f32),
}

#[derive(Clone, Debug)]
pub enum InstantaneousDrawCmd {
    PenDown,
    PenUp,
    PenColor(TurtleColor),
    FillColor(TurtleColor),
    PenWidth(f32),
    Dot(Option<f32>, TurtleColor),
    Stamp,
    Tracer(bool),
    BeginFill,
    EndFill,
    BeginPoly,
    EndPoly,
    SetDegrees(f32),
    Clear,
    Reset,
    Text(String),
}

#[derive(Clone, Debug)]
pub enum ScreenCmd {
    ClearScreen,
    Background(TurtleColor),
    BgPic(Vec<u8>),
    ClearStamp(usize),
    ClearStamps(isize),
    Speed(Speed),
    ShowTurtle(bool),
    SetSize([isize; 2]),
    RegisterShape(String, Shape),
    SetTitle(String),
    ExitOnClick,
    Bye,
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    KeyPress(fn(&mut Turtle, char), char),
    KeyRelease(fn(&mut Turtle, char), char),
    MousePress(fn(&mut Turtle, x: f32, y: f32)),
    MouseRelease(fn(&mut Turtle, x: f32, y: f32)),
    MouseDrag(fn(&mut Turtle, x: f32, y: f32)),
    Timer(fn(&mut Turtle, Duration), Duration),
}

// Commands which return data
#[derive(Clone, Debug)]
pub enum DataCmd {
    GetTurtles,
    GetShapes,
    GetScreenSize,
    GetPoly,
    GetPenState,
    GetFillingState,
    TurtleShape(TurtleShapeName),
    UndoBufferEntries,
    Towards(f32, f32),
    Position,
    Heading,
    Stamp,
    Visibility,
    TextInput(String, String), // title, prompt
    NumInput(String, String),  // title, prompt
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawRequest),
    Screen(ScreenCmd),
    Input(InputCmd),
    Data(DataCmd),
    Hatch,
    ShutDown,
}

impl DrawRequest {
    pub(crate) const fn is_stamp(&self) -> bool {
        matches!(self, Self::InstantaneousDraw(InstantaneousDrawCmd::Stamp))
    }

    pub(crate) const fn tracer_true(&self) -> bool {
        matches!(
            self,
            Self::InstantaneousDraw(InstantaneousDrawCmd::Tracer(true))
        )
    }

    pub(crate) const fn tracer_false(&self) -> bool {
        matches!(
            self,
            Self::InstantaneousDraw(InstantaneousDrawCmd::Tracer(false))
        )
    }
}
