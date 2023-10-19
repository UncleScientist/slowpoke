use piston::Key;

use crate::Turtle;

#[derive(Copy, Clone, Debug)]
pub enum DrawCmd {
    Forward(f64),
    Right(f64),
    Left(f64),
    PenDown,
    PenUp,
    GoTo(f64, f64),
    PenColor(f32, f32, f32),
    PenWidth(f64),
}

#[derive(Copy, Clone, Debug)]
pub enum ScreenCmd {
    ClearScreen,
    Background(f32, f32, f32),
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    OnKeyPress(fn(&mut Turtle, Key), Key),
}

#[derive(Copy, Clone, Debug)]
pub enum DataCmd {
    Position,
    Heading,
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Draw(DrawCmd),
    Screen(ScreenCmd),
    Input(InputCmd),
    Data(DataCmd),
}

impl DrawCmd {
    pub(crate) fn get_rotation(&self) -> f64 {
        match self {
            Self::Right(deg) => *deg,
            Self::Left(deg) => -*deg,
            _ => 0.,
        }
    }
}
