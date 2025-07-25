#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::f32::consts::PI;

use lyon_tessellation::{
    geom::euclid::{default::Point2D, default::Transform2D},
    math::Angle,
};

use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd, MotionCmd, RotateCmd, TimedDrawCmd},
    polygon::PolygonPath,
    ScreenPosition,
};

#[derive(Debug, Default, Clone)]
pub struct LineInfo {
    pub begin: ScreenPosition<i32>,
    pub end: ScreenPosition<i32>,
    pub pen_down: bool,
}

#[derive(Debug, Clone)]
pub struct CirclePos {
    pub angle: f32,
    pub x: i32,
    pub y: i32,
    pub pen_down: bool,
}

impl CirclePos {
    pub fn get_data(&self) -> (f32, [f32; 2]) {
        (self.angle, [self.x as f32, self.y as f32])
    }
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear,
    Reset,
    Filler,
    Filled(usize),
    BeginFill,
    EndFill,
    BeginPoly,
    EndPoly,
    StampTurtle,
    Line(LineInfo),
    SetPenColor(TurtleColor),
    SetPenWidth(f32),
    SetFillColor(TurtleColor),
    SetPosition(ScreenPosition<i32>),
    DrawPolygon(PolygonPath),
    SetHeading(f32, f32),
    Dot(Point2D<f32>, f32, TurtleColor), // center, radius, color
    DrawPolyAt(PolygonPath, ScreenPosition<f32>, f32), // poly, pos, angle
    Circle(Vec<CirclePos>),
    Text(Point2D<f32>, String),
}

impl DrawCommand {
    pub(crate) fn _needs_time(&self) -> bool {
        matches!(
            self,
            Self::Line(..) | Self::SetHeading(..) | Self::Circle(..)
        )
    }
}

#[derive(Debug)]
pub(crate) struct CurrentTurtleState {
    pub transform: Transform2D<f32>,
    pub angle: f32,
    pen_down: bool,
    pen_width: f32,
    fill_color: TurtleColor,
    pen_color: TurtleColor,
    circle_units: f32, // number of "degrees" in a circle
}

pub(crate) trait TurtlePosition<T> {
    fn pos(&self) -> ScreenPosition<T>;
}

impl TurtlePosition<f32> for CurrentTurtleState {
    fn pos(&self) -> ScreenPosition<f32> {
        self.transform
            .transform_point(ScreenPosition::new(0f32, 0f32))
    }
}

impl TurtlePosition<i32> for CurrentTurtleState {
    fn pos(&self) -> ScreenPosition<i32> {
        let point = self
            .transform
            .transform_point(ScreenPosition::new(0f32, 0f32));
        ScreenPosition::new(point.x as i32, point.y as i32)
    }
}

impl Default for CurrentTurtleState {
    fn default() -> Self {
        Self {
            pen_down: true,
            transform: Transform2D::identity(),
            angle: 0.,
            pen_width: 1.,
            pen_color: "black".into(),
            fill_color: "black".into(),
            circle_units: 360.,
        }
    }
}

impl CurrentTurtleState {
    pub fn angle(&self) -> f32 {
        self.angle
    }

    fn get_point(&self) -> ScreenPosition<i32> {
        let point: ScreenPosition<f32> = self.pos();
        [point.x.round() as i32, point.y.round() as i32].into()
    }

    fn get_floatpoint(&self) -> ScreenPosition<f32> {
        self.pos()
    }

    fn get_circlepos(&self) -> CirclePos {
        let point = self.get_point();
        CirclePos {
            angle: self.angle,
            x: point.x,
            y: point.y,
            pen_down: self.pen_down,
        }
    }

    pub(crate) fn get_state(&self) -> Vec<DrawCommand> {
        vec![
            DrawCommand::SetPosition(self.get_point()),
            DrawCommand::SetHeading(0., self.angle),
            DrawCommand::SetPenWidth(self.pen_width),
            DrawCommand::SetPenColor(self.pen_color),
            DrawCommand::SetFillColor(self.fill_color),
        ]
    }

    pub(crate) fn apply(&mut self, cmd: &DrawRequest) -> Option<DrawCommand> {
        match cmd {
            DrawRequest::TimedDraw(td) => match td {
                TimedDrawCmd::Circle(radius, extent, steps) => {
                    return Some(self.create_circle(*radius, *extent, *steps));
                }
                TimedDrawCmd::Motion(motion) => {
                    return Some(self.create_motion(motion));
                }
                TimedDrawCmd::Rotate(rotation) => {
                    let start = self.angle;
                    match rotation {
                        RotateCmd::Right(angle) => {
                            let angle = angle * (360. / self.circle_units);
                            let radians = Angle::degrees(angle);
                            self.transform = self.transform.pre_rotate(radians);
                            self.angle += angle;
                        }
                        RotateCmd::Left(angle) => {
                            let angle = angle * (360. / self.circle_units);
                            let radians = Angle::degrees(-angle);
                            self.transform = self.transform.pre_rotate(radians);
                            self.angle -= angle;
                        }
                        RotateCmd::SetHeading(h) => {
                            let h = h * (360. / self.circle_units);
                            let h = 180. - h;
                            let radians = Angle::degrees(h - self.angle + 90.);
                            self.transform = self.transform.pre_rotate(radians);
                            self.angle = h + 90.;
                        }
                    }
                    return Some(DrawCommand::SetHeading(start, self.angle));
                }
                TimedDrawCmd::Undo => {}
            },
            DrawRequest::InstantaneousDraw(id) => match id {
                InstantaneousDrawCmd::Reset => {
                    return Some(DrawCommand::Reset);
                }
                InstantaneousDrawCmd::Clear => {
                    return Some(DrawCommand::Clear);
                }
                InstantaneousDrawCmd::SetDegrees(deg) => {
                    self.circle_units = *deg;
                }
                InstantaneousDrawCmd::Tracer(_) => {}
                InstantaneousDrawCmd::PenDown => {
                    self.pen_down = true;
                }
                InstantaneousDrawCmd::PenUp => {
                    self.pen_down = false;
                }
                InstantaneousDrawCmd::PenColor(pc) => {
                    self.pen_color = *pc;
                    return Some(DrawCommand::SetPenColor(*pc));
                }
                InstantaneousDrawCmd::FillColor(fc) => {
                    self.fill_color = *fc;
                    return Some(DrawCommand::SetFillColor(*fc));
                }
                InstantaneousDrawCmd::PenWidth(pw) => {
                    self.pen_width = *pw / 2.;
                    return Some(DrawCommand::SetPenWidth(*pw / 2.));
                }
                InstantaneousDrawCmd::Dot(size, color) => {
                    let size = if let Some(size) = size {
                        *size
                    } else {
                        self.pen_width * 2.
                    };
                    let point = self.get_floatpoint();

                    let color = if matches!(color, TurtleColor::CurrentColor) {
                        self.fill_color
                    } else {
                        *color
                    };
                    return Some(DrawCommand::Dot(point, size, color));
                }
                InstantaneousDrawCmd::Stamp => {
                    return Some(DrawCommand::StampTurtle);
                }
                InstantaneousDrawCmd::BeginFill => return Some(DrawCommand::BeginFill),
                InstantaneousDrawCmd::EndFill => return Some(DrawCommand::EndFill),
                InstantaneousDrawCmd::BeginPoly => return Some(DrawCommand::BeginPoly),
                InstantaneousDrawCmd::EndPoly => return Some(DrawCommand::EndPoly),
                InstantaneousDrawCmd::Text(t) => {
                    let point = self.get_floatpoint();
                    return Some(DrawCommand::Text(point, t.clone()));
                }
            },
        }
        None
    }

    pub(crate) fn undo(&mut self, cmd: &DrawCommand) {
        match cmd {
            DrawCommand::Line(line) => {
                let x = line.begin.x as f32;
                let y = line.begin.y as f32;
                self.transform = Transform2D::translation(x, y);
            }
            DrawCommand::SetHeading(start, _) => {
                self.angle = *start;
            }
            _ => {}
        }
    }

    pub(crate) fn radians_to_turtle(&self, radians: f32) -> f32 {
        radians * (self.circle_units / (2. * PI))
    }

    pub(crate) fn degrees_to_turtle(&self, degrees: f32) -> f32 {
        degrees * (self.circle_units / 360.)
    }

    pub(crate) fn get_pen_state(&self) -> bool {
        self.pen_down
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    fn create_circle(&mut self, radius: f32, extent: f32, steps: usize) -> DrawCommand {
        let mut pointlist = vec![self.get_circlepos()];
        let rsign = -radius.signum();

        let extent = extent * (360. / self.circle_units);
        let theta_d = rsign * (extent / (steps as f32));
        let theta_r = rsign * (theta_d * (2. * PI / 360.));
        let len = 2. * radius.abs() * (theta_r / 2.).sin();

        let half_d: Angle = Angle::degrees(theta_d / 2.);
        let angle_d = Angle::degrees(theta_d);

        for s in 0..steps {
            if s == 0 {
                self.transform = self.transform.pre_rotate(half_d);
                self.angle += theta_d / 2.;
            } else {
                self.transform = self.transform.pre_rotate(angle_d);
                self.angle += theta_d;
            }

            self.transform = self.transform.pre_translate([len, 0.].into());
            pointlist.push(self.get_circlepos());
        }

        self.transform = self.transform.pre_rotate(half_d);
        self.angle += theta_d / 2.;

        DrawCommand::Circle(pointlist)
    }

    fn create_motion(&mut self, motion: &MotionCmd) -> DrawCommand {
        let begin = self.get_point();
        let start = self.get_floatpoint();
        let angle = Angle::degrees(self.angle);

        let mut pen_down = self.pen_down;
        match motion {
            MotionCmd::Forward(dist) => {
                self.transform = self.transform.pre_translate([*dist, 0.].into());
            }
            MotionCmd::Teleport(x, y) => {
                self.transform = Transform2D::translation(*x, *y).pre_rotate(angle);
                pen_down = false;
            }
            MotionCmd::GoTo(x, y) => {
                self.transform = Transform2D::translation(*x, *y).pre_rotate(angle);
            }
            MotionCmd::SetX(x) => {
                self.transform = Transform2D::translation(*x, start.y).then_rotate(angle);
            }
            MotionCmd::SetY(y) => {
                self.transform = Transform2D::translation(start.x, *y).then_rotate(angle);
            }
        }
        let end = self.get_point();

        DrawCommand::Line(LineInfo {
            begin,
            end,
            pen_down,
        })
    }
}
