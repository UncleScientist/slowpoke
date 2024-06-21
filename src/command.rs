use iced::Size;
use piston::Key;

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
    Circle(f64, f64, usize),
    Undo,
}

#[derive(Clone, Debug)]
pub enum MotionCmd {
    Forward(f64),
    GoTo(f64, f64),
    Teleport(f64, f64),
    SetX(f64),
    SetY(f64),
}

#[derive(Clone, Debug)]
pub enum RotateCmd {
    Right(f64),
    Left(f64),
    SetHeading(f64),
}

#[derive(Clone, Debug)]
pub enum InstantaneousDrawCmd {
    BackfillPolygon,
    PenDown,
    PenUp,
    PenColor(TurtleColor),
    FillColor(TurtleColor),
    PenWidth(f64),
    Dot(Option<f64>, TurtleColor),
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
    OnKeyPress(fn(&mut Turtle, Key), Key),
}

// Commands which return data
#[derive(Clone, Debug)]
pub enum DataCmd {
    GetScreenSize,
    GetPoly,
    TurtleShape(TurtleShapeName),
    UndoBufferEntries,
    Towards(f64, f64),
    Position,
    Heading,
    Stamp,
    Visibility,
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
        matches!(
            self,
            Self::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon)
                | Self::InstantaneousDraw(InstantaneousDrawCmd::Stamp)
        )
    }
}
