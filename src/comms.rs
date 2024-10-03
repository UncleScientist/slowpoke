use crate::{
    command::Command,
    turtle::types::{TurtleID, TurtleThread},
    Turtle,
};

#[derive(Debug)]
pub(crate) enum Response {
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

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) turtle: TurtleID,
    pub(crate) thread: TurtleThread,
    pub(crate) cmd: Command,
}

impl Request {
    pub(crate) fn shut_down(turtle: TurtleID, thread: TurtleThread) -> Request {
        Self {
            turtle,
            thread,
            cmd: Command::ShutDown,
        }
    }
}
