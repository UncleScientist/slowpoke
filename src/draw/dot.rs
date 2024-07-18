use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd},
    Turtle,
};

pub struct TurtleDot<'a> {
    width: Option<f32>,
    color: TurtleColor,
    turtle: &'a mut Turtle,
}

impl Turtle {
    pub fn dot(&mut self) -> TurtleDot {
        TurtleDot {
            width: None,
            color: TurtleColor::CurrentColor,
            turtle: self,
        }
    }
}

impl<'a> TurtleDot<'a> {
    pub fn with_size<S: Into<f64>>(mut self, size: S) -> TurtleDot<'a> {
        self.width = Some(size.into() as f32);
        self
    }

    pub fn with_color<C: Into<TurtleColor>>(mut self, color: C) -> TurtleDot<'a> {
        self.color = color.into();
        self
    }
}

impl<'a> Drop for TurtleDot<'a> {
    fn drop(&mut self) {
        self.turtle
            .do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Dot(
                self.width, self.color,
            )));
    }
}
