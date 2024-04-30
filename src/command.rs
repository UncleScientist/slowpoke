use piston::Key;

use crate::{
    color_names::TurtleColor,
    polygon::{TurtlePolygon, TurtleShapeName},
    speed::TurtleSpeed,
    Turtle,
};

#[derive(Clone, Debug)]
pub enum DrawCmd {
    TimedDraw(TimedDrawCmd),
    InstantaneousDraw(InstantaneousDrawCmd),
}

// commands that draw but don't return anything
#[derive(Clone, Debug)]
pub enum TimedDrawCmd {
    Motion(MotionCmd),
    Rotate(RotateCmd),
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
    Undo,
    BackfillPolygon,
    PenDown,
    PenUp,
    PenColor(TurtleColor),
    FillColor(TurtleColor),
    PenWidth(f64),
    Dot(Option<f64>, TurtleColor),
    Stamp(bool),
    Fill(TurtlePolygon),
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
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    OnKeyPress(fn(&mut Turtle, Key), Key),
}

// Commands which return data
#[derive(Clone, Debug)]
pub enum DataCmd {
    GetPoly,
    TurtleShape(TurtleShapeName),
    UndoBufferEntries,
    Towards(f64, f64),
    Position,
    Heading,
    Stamp,
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawCmd),
    Screen(ScreenCmd),
    Input(InputCmd),
    Data(DataCmd),
    Hatch,
}

impl DrawCmd {
    pub(crate) fn is_stamp(&self) -> bool {
        matches!(
            self,
            Self::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon)
                | Self::InstantaneousDraw(InstantaneousDrawCmd::Stamp(_))
        )
    }
}
