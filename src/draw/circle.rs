use crate::{
    command::{DrawRequest, TimedDrawCmd},
    Turtle,
};

pub struct TurtleCircle<'a> {
    radius: f64,
    steps: usize,
    extent: f64,
    turtle: &'a mut Turtle,
}

impl Turtle {
    pub fn circle<R: Into<f64>>(&mut self, radius: R) -> TurtleCircle {
        TurtleCircle {
            radius: radius.into(),
            steps: 32,
            extent: 360.,
            turtle: self,
        }
    }
}

impl<'a> TurtleCircle<'a> {
    pub fn with_steps(mut self, steps: usize) -> TurtleCircle<'a> {
        self.steps = steps;
        self
    }

    pub fn with_extent<E: Into<f64>>(mut self, extent: E) -> TurtleCircle<'a> {
        self.extent = extent.into();
        self
    }
}

impl<'a> Drop for TurtleCircle<'a> {
    fn drop(&mut self) {
        self.turtle
            .do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Circle(
                self.radius as f32,
                self.extent as f32,
                self.steps,
            )));
    }
}
