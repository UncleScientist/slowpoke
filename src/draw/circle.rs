use crate::{
    command::{DrawRequest, TimedDrawCmd},
    Turtle,
};

impl Turtle {
    pub fn extent<E: Into<f64>>(&mut self, extent: E) -> Extent {
        Extent::new(self, extent.into())
    }

    pub fn steps(&mut self, steps: usize) -> Steps {
        Steps::new(self, steps)
    }

    fn circle_full<R: Into<f64>, E: Into<f64>>(&mut self, radius: R, extent: E, steps: usize) {
        let radius = radius.into() as f32;
        let extent = extent.into() as f32;
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Circle(
            radius, extent, steps,
        )));
    }

    pub fn circle<R: Into<f64>>(&mut self, radius: R) {
        self.do_draw(DrawRequest::TimedDraw(TimedDrawCmd::Circle(
            radius.into() as f32,
            360.,
            42,
        )));
    }
}

pub struct Extent<'a> {
    extent: f64,
    turtle: &'a mut Turtle,
}

impl<'a> Extent<'a> {
    pub(crate) fn new(turtle: &'a mut Turtle, extent: f64) -> Self {
        Self { extent, turtle }
    }

    pub fn circle<R: Into<f64>>(&mut self, radius: R) {
        self.turtle.circle_full(radius.into(), self.extent, 20);
    }

    pub fn steps(self, steps: usize) -> ExtentAndSteps<'a> {
        ExtentAndSteps {
            extent: self.extent,
            steps,
            turtle: self.turtle,
        }
    }
}

pub struct Steps<'a> {
    steps: usize,
    turtle: &'a mut Turtle,
}

impl<'a> Steps<'a> {
    pub(crate) fn new(turtle: &'a mut Turtle, steps: usize) -> Self {
        Self { steps, turtle }
    }

    pub fn circle<R: Into<f64>>(&mut self, radius: R) {
        self.turtle.circle_full(radius.into(), 360., self.steps);
    }

    pub fn extent<E: Into<f64>>(self, extent: E) -> ExtentAndSteps<'a> {
        ExtentAndSteps {
            extent: extent.into(),
            steps: self.steps,
            turtle: self.turtle,
        }
    }
}

pub struct ExtentAndSteps<'a> {
    extent: f64,
    steps: usize,
    turtle: &'a mut Turtle,
}

impl<'a> ExtentAndSteps<'a> {
    pub fn circle<R: Into<f64>>(&mut self, radius: R) {
        self.turtle
            .circle_full(radius.into(), self.extent, self.steps);
    }
}
