use slowpoke::{SlowpokeLib, TurtleUserInterface};

pub type Slowpoke = SlowpokeLib<EguiFramework>;

#[derive(Debug)]
pub struct EguiFramework;

impl TurtleUserInterface for EguiFramework {
    fn start(_flags: slowpoke::TurtleFlags) {
        todo!()
    }
}
