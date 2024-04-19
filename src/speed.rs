#[derive(Debug, Copy, Clone)]
pub struct TurtleSpeed(u8);

impl Default for TurtleSpeed {
    fn default() -> Self {
        TurtleSpeed(3)
    }
}

impl TurtleSpeed {
    pub fn get(&self) -> u8 {
        self.0
    }
}

/*
impl<T> From<T> for TurtleSpeed
where
    T: Into<u8>,
{
    fn from(value: T) -> Self {
        Self(value.clamp(0, 10) as u8)
    }
}
*/

impl From<u8> for TurtleSpeed {
    fn from(value: u8) -> Self {
        Self(value.clamp(0, 10))
    }
}

impl From<i32> for TurtleSpeed {
    fn from(value: i32) -> Self {
        Self(value.clamp(0, 10) as u8)
    }
}

impl From<usize> for TurtleSpeed {
    fn from(value: usize) -> Self {
        Self(value.clamp(0, 10) as u8)
    }
}

impl From<&str> for TurtleSpeed {
    fn from(value: &str) -> Self {
        match value {
            "fastest" => Self(0),
            "fast" => Self(10),
            "normal" => Self(6),
            "slow" => Self(3),
            "slowest" => Self(1),
            _ => Self(5),
        }
    }
}
