use std::ops::{Deref, Index, IndexMut};

use super::TurtleData;

macro_rules! gen_generator {
    ($name:ident) => {
        #[derive(Debug, Hash, Eq, PartialEq, Default, Copy, Clone)]
        pub(crate) struct $name(IDGenerator);

        impl $name {
            pub(crate) fn new(id: usize) -> Self {
                Self(IDGenerator::new(id))
            }

            pub(crate) fn get(&mut self) -> Self {
                Self(self.0.get())
            }
        }
    };
}

#[derive(Debug, Hash, Eq, PartialEq, Default, Copy, Clone)]
pub(crate) struct IDGenerator {
    id: usize,
}

impl IDGenerator {
    pub(crate) fn get(&mut self) -> Self {
        self.id += 1;
        Self { id: self.id - 1 }
    }

    pub(crate) fn new(id: usize) -> Self {
        Self { id }
    }
}

gen_generator!(TurtleID);
gen_generator!(TurtleThread);

#[cfg(feature = "ratatui")]
gen_generator!(PopupID);

impl From<usize> for TurtleID {
    fn from(id: usize) -> Self {
        Self(IDGenerator::new(id))
    }
}

impl Deref for TurtleID {
    type Target = IDGenerator;

    fn deref(&self) -> &Self::Target {
        &self.0
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
