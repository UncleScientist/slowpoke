use iced::Size;

use crate::{
    color_names::TurtleColor,
    polygon::{TurtlePolygon, TurtleShapeName},
    speed::TurtleSpeed,
    Turtle,
};

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
    BackfillPolygon,
    PenDown,
    PenUp,
    PenColor(TurtleColor),
    FillColor(TurtleColor),
    PenWidth(f32),
    Dot(Option<f32>, TurtleColor),
    Stamp,
    Fill(TurtlePolygon),
    Tracer(bool),
}

#[derive(Copy, Clone, Debug)]
pub enum ScreenCmd {
    ClearScreen,
    Background(TurtleColor),
    ClearStamp(usize),
    ClearStamps(isize),
    BeginFill,
    EndFill,
    BeginPoly,
    EndPoly,
    Speed(TurtleSpeed),
    ShowTurtle(bool),
    SetSize(Size),
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    OnKeyPress(fn(&mut Turtle, char), char),
}

// Commands which return data
#[derive(Clone, Debug)]
pub enum DataCmd {
    GetScreenSize,
    GetPoly,
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
}

impl DrawRequest {
    pub(crate) fn is_stamp(&self) -> bool {
        matches!(self, Self::InstantaneousDraw(InstantaneousDrawCmd::Stamp))
    }

    pub(crate) fn tracer_true(&self) -> bool {
        matches!(
            self,
            DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(true))
        )
    }

    pub(crate) fn tracer_false(&self) -> bool {
        matches!(
            self,
            DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(false))
        )
    }
}
