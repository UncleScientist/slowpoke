#[derive(Debug, Copy, Clone)]
pub struct Speed(u8);

impl Default for Speed {
    fn default() -> Self {
        Speed(3)
    }
}

impl Speed {
    #[must_use]
    pub fn get(&self) -> u8 {
        self.0
    }
}

/* TODO: investigate using the num_traits crate
impl<T> From<T> for TurtleSpeed
where
    T: Into<u8>,
{
    fn from(value: T) -> Self {
        Self(value.clamp(0, 10) as u8)
    }
}
*/

impl From<u8> for Speed {
    fn from(value: u8) -> Self {
        Self(value.clamp(0, 10))
    }
}

impl From<i32> for Speed {
    fn from(value: i32) -> Self {
        Self((value.clamp(0, 10).unsigned_abs() & 0xff) as u8)
    }
}

impl From<usize> for Speed {
    fn from(value: usize) -> Self {
        Self(u8::try_from(value.clamp(0, 10)).expect("try_from failure"))
    }
}

impl From<&str> for Speed {
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
