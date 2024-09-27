use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd},
    Turtle,
};

pub struct TurtleDotProps<'a> {
    width: Option<f32>,
    color: TurtleColor,
    turtle: &'a mut Turtle,
}

impl Turtle {
    pub fn dot(&mut self) -> TurtleDotProps {
        TurtleDotProps {
            width: None,
            color: TurtleColor::CurrentColor,
            turtle: self,
        }
    }
}

impl<'a> TurtleDotProps<'a> {
    #[allow(clippy::cast_possible_truncation)]
    pub fn with_size<S: Into<f64>>(mut self, size: S) -> TurtleDotProps<'a> {
        self.width = Some(size.into() as f32);
        self
    }

    pub fn with_color<C: Into<TurtleColor>>(mut self, color: C) -> TurtleDotProps<'a> {
        self.color = color.into();
        self
    }
}

impl<'a> Drop for TurtleDotProps<'a> {
    fn drop(&mut self) {
        self.turtle
            .do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Dot(
                self.width, self.color,
            )));
    }
}
