use crate::{
    command::Command,
    turtle::types::{TurtleID, TurtleThread},
    Turtle,
};

pub enum Response {
    Done,
    Cancel,
    Heading(f32),
    Position(crate::ScreenPosition<i32>),
    StampID(crate::StampID),
    Turtle(crate::Turtle),
    Count(usize),
    Name(String),
    Polygon(Vec<[f32; 2]>),
    Visibility(bool),
    ScreenSize([isize; 2]),
    TextInput(String),
    NumInput(f32),
    IsPenDown(bool),
    IsFilling(bool),
    ShapeList(Vec<String>),
    Turtles(Vec<Turtle>),
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Response::Done => todo!(),
                Response::Cancel => todo!(),
                Response::Heading(_) => todo!(),
                Response::Position(point2_d) => todo!(),
                Response::StampID(_) => todo!(),
                Response::Turtle(turtle) => todo!(),
                Response::Count(_) => todo!(),
                Response::Name(_) => todo!(),
                Response::Polygon(vec) => todo!(),
                Response::Visibility(_) => todo!(),
                Response::ScreenSize(_) => todo!(),
                Response::TextInput(_) => todo!(),
                Response::NumInput(_) => todo!(),
                Response::IsPenDown(_) => todo!(),
                Response::IsFilling(_) => todo!(),
                Response::ShapeList(vec) => todo!(),
                Response::Turtles(vec) => todo!(),
            }
        )
    }
}

#[derive(Debug)]
pub struct Request {
    pub turtle: TurtleID,
    pub thread: TurtleThread,
    pub cmd: Command,
}

impl Request {
    pub(crate) const fn shut_down(turtle: TurtleID, thread: TurtleThread) -> Self {
        Self {
            turtle,
            thread,
            cmd: Command::ShutDown,
        }
    }
}
