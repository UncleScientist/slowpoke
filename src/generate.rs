use graphics::{
    math::{identity, Vec2d},
    types::Rectangle,
    Transformed,
};

use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd, MotionCmd, RotateCmd, TimedDrawCmd},
    polygon::TurtlePolygon,
};

#[derive(Debug)]
pub(crate) struct LineInfo {
    pub begin: Vec2d<isize>,
    pub end: Vec2d<isize>,
    pub pen_down: bool,
}

#[derive(Debug)]
pub(crate) enum DrawCommand {
    Filler,
    DrawLine(LineInfo),
    SetPenColor(TurtleColor),
    SetPenWidth(f64),
    SetFillColor(TurtleColor),
    DrawPolygon(TurtlePolygon),
    SetHeading(f64, f64),
    DrawDot(Rectangle, TurtleColor),
}

#[derive(Debug)]
pub(crate) struct CurrentTurtleState {
    pub transform: [[f64; 3]; 2],
    pub angle: f64,
    pen_down: bool,
    pen_width: f64,
    fill_color: TurtleColor,
}

pub(crate) trait TurtlePosition<T> {
    fn pos(&self) -> [T; 2];
}

impl TurtlePosition<f64> for CurrentTurtleState {
    fn pos(&self) -> [f64; 2] {
        [self.transform[0][2], self.transform[1][2]]
    }
}

impl TurtlePosition<isize> for CurrentTurtleState {
    fn pos(&self) -> [isize; 2] {
        [self.transform[0][2] as isize, self.transform[1][2] as isize]
    }
}

impl Default for CurrentTurtleState {
    fn default() -> Self {
        Self {
            pen_down: true,
            transform: identity(),
            angle: 0.,
            pen_width: 0.5,
            fill_color: "black".into(),
        }
    }
}

impl CurrentTurtleState {
    pub fn angle(&self) -> f64 {
        self.angle
    }

    fn get_point(&self) -> Vec2d<isize> {
        let x = self.transform[0][2].round() as isize;
        let y = self.transform[1][2].round() as isize;
        [x, y]
    }

    fn get_floatpoint(&self) -> Vec2d<f64> {
        [self.transform[0][2], self.transform[1][2]]
    }

    pub(crate) fn apply(&mut self, cmd: &DrawRequest) -> Option<DrawCommand> {
        match cmd {
            DrawRequest::TimedDraw(td) => match td {
                TimedDrawCmd::Motion(motion) => {
                    let begin = self.get_point();
                    let mut pen_down = self.pen_down;
                    match motion {
                        MotionCmd::Forward(dist) => {
                            self.transform = self.transform.trans(*dist, 0.);
                        }
                        MotionCmd::Teleport(x, y) => {
                            self.transform = identity().trans(*x, *y).rot_deg(self.angle);
                            pen_down = false;
                        }
                        MotionCmd::GoTo(x, y) => {
                            self.transform = identity().trans(*x, *y).rot_deg(self.angle);
                        }
                        MotionCmd::SetX(x) => {
                            let cur_y = self.transform[1][2];
                            self.transform = identity().trans(*x, cur_y).rot_deg(self.angle);
                        }
                        MotionCmd::SetY(y) => {
                            let cur_x = self.transform[0][2];
                            self.transform = identity().trans(cur_x, *y).rot_deg(self.angle);
                        }
                    }
                    let end = self.get_point();
                    return Some(DrawCommand::DrawLine(LineInfo {
                        begin,
                        end,
                        pen_down,
                    }));
                }
                TimedDrawCmd::Rotate(rotation) => {
                    let start = self.angle;
                    match rotation {
                        RotateCmd::Right(angle) => {
                            self.transform = self.transform.rot_deg(*angle);
                            self.angle += angle;
                        }
                        RotateCmd::Left(angle) => {
                            self.transform = self.transform.rot_deg(-*angle);
                            self.angle -= angle;
                        }
                        RotateCmd::SetHeading(h) => {
                            self.transform = self.transform.rot_deg(h - self.angle);
                            self.angle = *h;
                        }
                    }
                    return Some(DrawCommand::SetHeading(start, self.angle));
                }
            },
            DrawRequest::InstantaneousDraw(id) => match id {
                InstantaneousDrawCmd::Undo => {}
                InstantaneousDrawCmd::BackfillPolygon => return Some(DrawCommand::Filler),
                InstantaneousDrawCmd::PenDown => {
                    self.pen_down = true;
                }
                InstantaneousDrawCmd::PenUp => {
                    self.pen_down = false;
                }
                InstantaneousDrawCmd::PenColor(pc) => {
                    return Some(DrawCommand::SetPenColor(*pc));
                }
                InstantaneousDrawCmd::FillColor(fc) => {
                    return Some(DrawCommand::SetFillColor(*fc));
                }
                InstantaneousDrawCmd::PenWidth(pw) => {
                    return Some(DrawCommand::SetPenWidth(*pw));
                }
                InstantaneousDrawCmd::Dot(size, color) => {
                    let size = if let Some(size) = size {
                        *size
                    } else {
                        self.pen_width
                    };
                    let point: [f64; 2] = self.get_floatpoint();
                    let rect = [point[0] - size / 2., point[1] - size / 2., size, size];
                    let color = if matches!(color, TurtleColor::CurrentColor) {
                        self.fill_color
                    } else {
                        *color
                    };
                    return Some(DrawCommand::DrawDot(rect, color));
                }
                InstantaneousDrawCmd::Stamp(_) => {}
                InstantaneousDrawCmd::Fill(polygon) => {
                    return Some(DrawCommand::DrawPolygon(polygon.clone()));
                }
            },
        }
        None
    }
}