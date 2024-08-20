use std::ops::Deref;

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

impl Deref for TurtleID {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.id
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
}

impl Deref for TurtleThread {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.thread
    }
}
