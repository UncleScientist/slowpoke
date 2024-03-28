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

impl From<u8> for TurtleSpeed {
    fn from(value: u8) -> Self {
        if value > 10 {
            Self(10)
        } else {
            Self(value)
        }
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
