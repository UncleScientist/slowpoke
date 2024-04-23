use graphics::{Context, Transformed};
use opengl_graphics::GlGraphics;
use piston::Key;

use crate::{
    color_names::TurtleColor,
    polygon::{TurtlePolygon, TurtleShapeName},
    speed::TurtleSpeed,
    Turtle,
};

#[derive(Clone, Debug)]
pub enum DrawCmd {
    TimedDraw(TimedDrawCmd),
    InstantaneousDraw(InstantaneousDrawCmd),
}

// commands that draw but don't return anything
#[derive(Clone, Debug)]
pub enum TimedDrawCmd {
    Forward(f64),
    Right(f64),
    Left(f64),
    GoTo(f64, f64),
    Teleport(f64, f64),
    SetX(f64),
    SetY(f64),
    SetHeading(f64),
}

#[derive(Clone, Debug)]
pub enum InstantaneousDrawCmd {
    Undo,
    BackfillPolygon,
    PenDown,
    PenUp,
    PenColor(TurtleColor),
    FillColor(TurtleColor),
    PenWidth(f64),
    Dot(Option<f64>, TurtleColor),
    Stamp(bool),
    Fill(TurtlePolygon),
}

pub(crate) struct TurtleDrawState<'a> {
    pub context: Context,
    pub x: f64,
    pub y: f64,
    pub size: [f64; 2],
    pub transform: [[f64; 3]; 2],
    pub win_center: [[f64; 3]; 2],
    pub pct: f64,
    pub deg: f64,
    pub start_deg: f64,
    pub pen_color: [f32; 4],
    pub fill_color: [f32; 4],
    pub pen_width: f64,
    pub is_pen_down: bool,
    pub gl: &'a mut GlGraphics,
    pub shape: TurtlePolygon,
}

impl<'a> TurtleDrawState<'a> {
    pub(crate) fn new(
        size: [f64; 2],
        context: Context,
        gl: &'a mut GlGraphics,
        shape: TurtlePolygon,
    ) -> Self {
        let (x, y) = (size[0] / 2.0, size[1] / 2.0);
        let win_center = context.transform.trans(x, y);
        Self {
            context,
            x,
            y,
            size,
            is_pen_down: true,
            transform: win_center,
            win_center,
            pct: 1.,
            deg: 0.,
            start_deg: 0.,
            pen_color: crate::BLACK,
            fill_color: crate::BLACK,
            pen_width: 0.5,
            gl,
            shape,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScreenCmd {
    ClearScreen,
    Background(TurtleColor),
    ClearStamp(usize),
    ClearStamps(isize),
    BeginFill,
    EndFill,
    BeginPoly,
    EndPoly,
    Speed(TurtleSpeed),
}

#[derive(Copy, Clone, Debug)]
pub enum InputCmd {
    OnKeyPress(fn(&mut Turtle, Key), Key),
}

// Commands which return data
#[derive(Clone, Debug)]
pub enum DataCmd {
    GetPoly,
    TurtleShape(TurtleShapeName),
    UndoBufferEntries,
    Towards(f64, f64),
    Position,
    Heading,
    Stamp,
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawCmd),
    Screen(ScreenCmd),
    Input(InputCmd),
    Data(DataCmd),
    Hatch,
}

impl InstantaneousDrawCmd {
    fn draw(&self, ds: &mut TurtleDrawState) {
        match self {
            Self::BackfillPolygon | Self::Undo => {}
            Self::PenDown => ds.is_pen_down = true,
            Self::PenUp => ds.is_pen_down = false,
            Self::PenColor(TurtleColor::Color(r, g, b)) => {
                ds.pen_color = [*r, *g, *b, 1.];
            }
            Self::FillColor(TurtleColor::Color(r, g, b)) => {
                ds.fill_color = [*r, *g, *b, 1.];
            }
            Self::PenColor(TurtleColor::CurrentColor)
            | Self::FillColor(TurtleColor::CurrentColor) => {}
            Self::PenWidth(width) => {
                ds.pen_width = *width;
            }
            Self::Dot(width, color) => {
                let default_width = (ds.pen_width * 2.).max(ds.pen_width + 4.);
                let width = width.unwrap_or(default_width);
                let color = if let TurtleColor::Color(r, g, b) = color {
                    [*r, *g, *b, 1.]
                } else {
                    ds.pen_color
                };
                graphics::ellipse(
                    color,
                    [-width / 2., -width / 2., width, width],
                    ds.transform,
                    ds.gl,
                );
            }
            Self::Stamp(draw) => {
                if *draw {
                    let x = ds.shape.clone();
                    x.draw(&ds.pen_color.clone(), ds.transform, ds);
                }
            }
            InstantaneousDrawCmd::Fill(poly) => {
                poly.draw(&ds.fill_color.clone(), ds.win_center.flip_v(), ds);
            }
        };
    }
}

impl TimedDrawCmd {
    fn get_rotation(&self, ds: &TurtleDrawState) -> f64 {
        match self {
            Self::Right(deg) => *deg,
            Self::Left(deg) => -*deg,
            Self::SetHeading(deg) => *deg - ds.deg,
            _ => 0.,
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

    fn draw(&self, ds: &mut TurtleDrawState) {
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
            Self::GoTo(xpos, ypos) => self.move_to(ds, *xpos, *ypos),
            Self::Teleport(xpos, ypos) => {
                let saved_pen = ds.is_pen_down;
                ds.is_pen_down = false;
                self.move_to(ds, *xpos, *ypos);
                ds.is_pen_down = saved_pen;
            }
            Self::SetX(xpos) => {
                let ypos = -ds.transform[1][2] * ds.size[1] / 2.;
                self.move_to(ds, *xpos, ypos);
            }
            Self::SetY(ypos) => {
                let xpos = ds.transform[0][2] * ds.size[0] / 2.;
                self.move_to(ds, xpos, *ypos);
            }
        }
    }
}

impl DrawCmd {
    pub(crate) fn is_stamp(&self) -> bool {
        matches!(
            self,
            Self::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon)
                | Self::InstantaneousDraw(InstantaneousDrawCmd::Stamp(_))
        )
    }

    pub(crate) fn get_rotation(&self, ds: &TurtleDrawState) -> f64 {
        match self {
            DrawCmd::TimedDraw(t) => t.get_rotation(ds),
            DrawCmd::InstantaneousDraw(_) => 0.,
        }
    }

    pub(crate) fn draw(&self, ds: &mut TurtleDrawState) {
        match self {
            DrawCmd::TimedDraw(t) => t.draw(ds),
            DrawCmd::InstantaneousDraw(i) => i.draw(ds),
        }
    }
}
