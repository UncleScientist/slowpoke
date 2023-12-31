use graphics::{Context, Transformed};
use opengl_graphics::GlGraphics;
use piston::Key;

use crate::Turtle;

#[derive(Copy, Clone, Debug)]
pub enum DrawCmd {
    Forward(f64),
    Right(f64),
    Left(f64),
    PenDown,
    PenUp,
    GoTo(f64, f64),
    SetX(f64),
    SetY(f64),
    SetHeading(f64),
    PenColor(f32, f32, f32),
    PenWidth(f64),
}

pub(crate) struct TurtleDrawState<'a> {
    pub context: Context,
    pub x: f64,
    pub y: f64,
    pub size: [f64; 2],
    pub transform: [[f64; 3]; 2],
    pub pct: f64,
    pub deg: f64,
    pub start_deg: f64,
    pub pen_color: [f32; 4],
    pub pen_width: f64,
    pub is_pen_down: bool,
    pub gl: &'a mut GlGraphics,
}

#[derive(Copy, Clone, Debug)]
pub enum ScreenCmd {
    ClearScreen,
    Background(f32, f32, f32),
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    OnKeyPress(fn(&mut Turtle, Key), Key),
}

#[derive(Copy, Clone, Debug)]
pub enum DataCmd {
    Position,
    Heading,
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Draw(DrawCmd),
    Screen(ScreenCmd),
    Input(InputCmd),
    Data(DataCmd),
}

impl DrawCmd {
    pub(crate) fn get_rotation(&self, ds: &TurtleDrawState) -> f64 {
        match self {
            Self::Right(deg) => *deg,
            Self::Left(deg) => -*deg,
            Self::SetHeading(deg) => *deg - ds.deg,
            _ => 0.,
        }
    }

    pub(crate) fn draw(&self, ds: &mut TurtleDrawState) {
        match self {
            Self::Forward(dist) => {
                if ds.is_pen_down {
                    graphics::line_from_to(
                        ds.pen_color,
                        ds.pen_width,
                        [0., 0.],
                        [dist * ds.pct, 0.],
                        ds.transform,
                        ds.gl,
                    );
                }
                ds.transform = ds.transform.trans(dist * ds.pct, 0.);
            }
            Self::Right(deg) => ds.transform = ds.transform.rot_deg(deg * ds.pct),
            Self::Left(deg) => ds.transform = ds.transform.rot_deg(-deg * ds.pct),
            Self::SetHeading(heading) => {
                ds.transform = ds.transform.rot_deg(heading - ds.start_deg)
            }
            Self::PenDown => ds.is_pen_down = true,
            Self::PenUp => ds.is_pen_down = false,
            Self::GoTo(xpos, ypos) => self.move_to(ds, *xpos, *ypos),
            Self::SetX(xpos) => {
                let ypos = -ds.transform[1][2] * ds.size[1] / 2.;
                self.move_to(ds, *xpos, ypos);
            }
            Self::SetY(ypos) => {
                let xpos = ds.transform[0][2] * ds.size[0] / 2.;
                self.move_to(ds, xpos, *ypos);
            }
            Self::PenColor(r, g, b) => {
                ds.pen_color = [*r, *g, *b, 1.];
            }
            Self::PenWidth(width) => {
                ds.pen_width = *width;
            }
        }
    }

    // move to absolute coordinates, drawing a line if the pen is down
    fn move_to(&self, ds: &mut TurtleDrawState, xpos: f64, ypos: f64) {
        let dest_x = xpos;
        let dest_y = ypos;
        let cur_x = ds.transform[0][2] * ds.size[0] / 2.;
        let cur_y = -ds.transform[1][2] * ds.size[1] / 2.;
        let pct_x = cur_x + (dest_x - cur_x) * ds.pct;
        let pct_y = cur_y + (dest_y - cur_y) * ds.pct;
        if ds.is_pen_down {
            graphics::line_from_to(
                ds.pen_color,
                ds.pen_width,
                [ds.x + cur_x, ds.y + cur_y],
                [ds.x + pct_x, ds.y + pct_y],
                ds.context.transform,
                ds.gl,
            );
        }
        ds.transform = ds
            .context
            .transform
            .trans(pct_x + ds.x, pct_y + ds.y)
            .rot_deg(ds.deg);
    }
}
