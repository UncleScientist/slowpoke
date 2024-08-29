use std::ops::{Index, IndexMut};

use super::TurtleData;

#[derive(Debug, Hash, Eq, PartialEq, Default, Copy, Clone)]
pub(crate) struct TurtleID {
    id: usize,
}
impl TurtleID {
    pub(crate) fn get(&mut self) -> Self {
        self.id += 1;
        Self { id: self.id - 1 }
    }

    pub(crate) fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Default, Copy, Clone)]
pub(crate) struct TurtleThread {
    thread: usize,
}

impl TurtleThread {
    pub(crate) fn new(thread: usize) -> Self {
        Self { thread }
    }

    pub(crate) fn get(&mut self) -> Self {
        self.thread += 1;
        Self {
            thread: self.thread - 1,
        }
    }
}

impl Index<TurtleID> for Vec<TurtleData> {
    type Output = TurtleData;

    fn index(&self, index: TurtleID) -> &Self::Output {
        &self[index.id]
    }
}

impl IndexMut<TurtleID> for Vec<TurtleData> {
    fn index_mut(&mut self, index: TurtleID) -> &mut Self::Output {
        &mut self[index.id]
    }
}
