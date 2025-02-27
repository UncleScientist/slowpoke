use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd},
    turtle::TurtleUserInterface,
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

impl TurtleDotProps<'_> {
    #[allow(clippy::cast_possible_truncation)]
    pub fn with_size<S: Into<f64>>(mut self, size: S) -> Self {
        self.width = Some(size.into() as f32);
        self
    }

    pub fn with_color<C: Into<TurtleColor>>(mut self, color: C) -> Self {
        self.color = color.into();
        self
    }
}

impl Drop for TurtleDotProps<'_> {
    fn drop(&mut self) {
        self.turtle
            .do_draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Dot(
                self.width, self.color,
            )));
    }
}
