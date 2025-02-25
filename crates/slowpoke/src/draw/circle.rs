use crate::{
    command::{DrawRequest, TimedDrawCmd},
    Turtle,
};

pub struct TurtleCircleProps<'a> {
    radius: f64,
    steps: usize,
    extent: f64,
    turtle: &'a mut Turtle,
}

impl Turtle {
    pub fn circle<R: Into<f64>>(&mut self, radius: R) -> TurtleCircleProps {
        TurtleCircleProps {
            radius: radius.into(),
            steps: 32,
            extent: 360.,
            turtle: self,
        }
    }
}

impl TurtleCircleProps<'_> {
    pub fn with_steps<S: Into<usize>>(mut self, steps: S) -> Self {
        self.steps = steps.into();
        self
    }

    pub fn with_extent<E: Into<f64>>(mut self, extent: E) -> Self {
        self.extent = extent.into();
        self
    }
}

impl Drop for TurtleCircleProps<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn drop(&mut self) {
        self.turtle
            .do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Circle(
                self.radius as f32,
                self.extent as f32,
                self.steps,
            )));
    }
}
