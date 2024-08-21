use std::f32::consts::PI;

use lyon_tessellation::{
    geom::euclid::{default::Point2D, default::Transform2D},
    math::Angle,
};

use crate::{
    color_names::TurtleColor,
    command::{DrawRequest, InstantaneousDrawCmd, MotionCmd, RotateCmd, TimedDrawCmd},
    polygon::TurtlePolygon,
    ScreenPosition,
};

#[derive(Debug, Clone)]
pub(crate) struct LineInfo {
    pub begin: ScreenPosition<isize>,
    pub end: ScreenPosition<isize>,
    pub pen_down: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CirclePos {
    pub angle: f32,
    pub x: isize,
    pub y: isize,
    pub pen_down: bool,
}

impl CirclePos {
    pub fn get_data(&self) -> (f32, [f32; 2]) {
        (self.angle, [self.x as f32, self.y as f32])
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DrawCommand {
    Filler,
    BeginFill,
    EndFill,
    BeginPoly,
    EndPoly,
    StampTurtle,
    Line(LineInfo),
    SetPenColor(TurtleColor),
    SetPenWidth(f32),
    SetFillColor(TurtleColor),
    DrawPolygon(TurtlePolygon),
    SetHeading(f32, f32),
    DrawDot(Point2D<f32>, f32, TurtleColor), // center, radius, color
    DrawPolyAt(TurtlePolygon, ScreenPosition<f32>, f32), // poly, pos, angle
    Circle(Vec<CirclePos>),
}

impl DrawCommand {
    pub(crate) fn is_stamp(&self) -> bool {
        matches!(self, Self::DrawPolyAt(..))
    }
}

#[derive(Debug)]
pub(crate) struct CurrentTurtleState {
    pub transform: Transform2D<f32>,
    pub angle: f32,
    pen_down: bool,
    pen_width: f32,
    fill_color: TurtleColor,
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

impl TurtlePosition<isize> for CurrentTurtleState {
    fn pos(&self) -> ScreenPosition<isize> {
        let point = self
            .transform
            .transform_point(ScreenPosition::new(0f32, 0f32));
        ScreenPosition::new(point.x as isize, point.y as isize)
    }
}

impl Default for CurrentTurtleState {
    fn default() -> Self {
        Self {
            pen_down: true,
            transform: Transform2D::identity(),
            angle: 0.,
            pen_width: 1.,
            fill_color: "black".into(),
        }
    }
}

impl CurrentTurtleState {
    pub fn angle(&self) -> f32 {
        self.angle
    }

    fn get_point(&self) -> ScreenPosition<isize> {
        let point: ScreenPosition<f32> = self.pos();
        [point.x.round() as isize, point.y.round() as isize].into()
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

    pub(crate) fn apply(&mut self, cmd: &DrawRequest) -> Option<DrawCommand> {
        match cmd {
            DrawRequest::TimedDraw(td) => match td {
                TimedDrawCmd::Circle(radius, extent, steps) => {
                    let mut pointlist = vec![self.get_circlepos()];
                    let rsign = -radius.signum();

                    let theta_d = rsign * (extent / (*steps as f32));
                    let theta_r = rsign * (theta_d * (2. * PI / 360.));
                    let len = 2. * radius.abs() * (theta_r / 2.).sin();

                    let half_d: Angle = Angle::degrees(theta_d / 2.);
                    let angle_d = Angle::degrees(theta_d);

                    for s in 0..*steps {
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
                    return Some(DrawCommand::Circle(pointlist));
                }
                TimedDrawCmd::Motion(motion) => {
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
                            self.transform =
                                Transform2D::translation(*x, start.y).then_rotate(angle);
                        }
                        MotionCmd::SetY(y) => {
                            self.transform =
                                Transform2D::translation(start.x, *y).then_rotate(angle);
                        }
                    }
                    let end = self.get_point();
                    return Some(DrawCommand::Line(LineInfo {
                        begin,
                        end,
                        pen_down,
                    }));
                }
                TimedDrawCmd::Rotate(rotation) => {
                    let start = self.angle;
                    match rotation {
                        RotateCmd::Right(angle) => {
                            let radians = Angle::degrees(*angle);
                            self.transform = self.transform.pre_rotate(radians);
                            self.angle += angle;
                        }
                        RotateCmd::Left(angle) => {
                            let radians = Angle::degrees(-*angle);
                            self.transform = self.transform.pre_rotate(radians);
                            self.angle -= angle;
                        }
                        RotateCmd::SetHeading(h) => {
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
                InstantaneousDrawCmd::Tracer(_) => {}
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
                    return Some(DrawCommand::DrawDot(point, size, color));
                }
                InstantaneousDrawCmd::Stamp => {
                    return Some(DrawCommand::StampTurtle);
                }
                InstantaneousDrawCmd::BeginFill => return Some(DrawCommand::BeginFill),
                InstantaneousDrawCmd::EndFill => return Some(DrawCommand::EndFill),
                InstantaneousDrawCmd::BeginPoly => return Some(DrawCommand::BeginPoly),
                InstantaneousDrawCmd::EndPoly => return Some(DrawCommand::EndPoly),
            },
        }
        None
    }
}
