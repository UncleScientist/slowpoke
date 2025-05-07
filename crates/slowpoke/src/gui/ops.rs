use lyon_tessellation::{
    geom::euclid::{default::Point2D, default::Transform2D},
    math::Angle,
};

use crate::{polygon::PolygonPath, CirclePos, IndividualTurtle};
use crate::{DrawCommand, LineInfo, TurtleColor};

type Point = Point2D<f32>;

#[derive(Debug)]
pub enum DrawOp {
    ScreenCmd(ScreenDraw),
    TurtleCmd(TurtleDraw),
}

#[derive(Debug)]
pub enum ScreenDraw {
    SetBackgroundColor,
    SetBackgroundImage,
    SetScreenSize,
    SetTitle,
    UserEvent,
    TimerTick,
    PopupNumber,
    PopupText,
    GetScreenSize,
    GetImage,
}

#[derive(Debug)]
pub enum TurtleDraw {
    _DrawLine(LineSegment),
    DrawLines(TurtleColor, f32, Vec<LineSegment>),
    DrawDot(Point, f32, TurtleColor),
    DrawText(Point, String),
    FillPolygon(TurtleColor, TurtleColor, f32, Vec<LineSegment>),
    _SetLineWidth,
    _SetLineColor,
    _SetFillColor,
}

#[derive(Debug)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}
impl LineSegment {
    fn transform(&self, xform: &Transform2D<f32>) -> LineSegment {
        let start = xform.transform_point(self.start);
        let end = xform.transform_point(self.end);
        LineSegment { start, end }
    }
}

impl TurtleDraw {
    pub(crate) fn convert<UI>(pct: f32, turtle: &IndividualTurtle<UI>) -> Vec<Self> {
        fn make_path(path: &mut Vec<(bool, Point)>) -> Vec<LineSegment> {
            let mut segments = Vec::new();
            let mut cur_pos = path.remove(0).1;
            for (pen, pos) in path.drain(..) {
                if pen {
                    segments.push(LineSegment {
                        start: cur_pos,
                        end: pos,
                    });
                }
                cur_pos = pos;
            }
            segments
        }

        let mut drawing = Vec::new();

        let mut pencolor = TurtleColor::Color(0., 0., 0.);
        let mut penwidth = 1.0;
        let mut fillcolor = TurtleColor::Color(0., 0., 0.);

        let mut tpos = [0f32, 0f32];
        let mut trot = 0f32;

        let mut iter = turtle.cmds.iter().peekable();
        let mut cur_path: Vec<(bool, Point)> = Vec::new();

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            if !matches!(element, DrawCommand::Line(..))
                && !matches!(element, DrawCommand::SetHeading(..))
                && !cur_path.is_empty()
            {
                drawing.push(TurtleDraw::DrawLines(
                    pencolor,
                    penwidth,
                    make_path(&mut cur_path),
                ));
            }

            match element {
                DrawCommand::Line(line) => {
                    let (start, end) = Self::start_and_end(last_element, pct, line);
                    tpos = [end.x, end.y];
                    if cur_path.is_empty() {
                        cur_path.push((line.pen_down, start));
                    }
                    cur_path.push((line.pen_down, end));
                }
                DrawCommand::SetPenColor(pc) => {
                    pencolor = *pc;
                }
                DrawCommand::SetPenWidth(pw) => penwidth = *pw,
                DrawCommand::SetFillColor(fc) => {
                    fillcolor = *fc;
                }
                DrawCommand::DrawPolygon(p) => {
                    drawing.push(TurtleDraw::FillPolygon(
                        fillcolor,
                        pencolor,
                        penwidth,
                        p.get_path(),
                    ));
                }
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * pct
                    } else {
                        *end
                    };
                    trot = rotation;
                }
                DrawCommand::Dot(center, radius, color) => {
                    let center: Point = Point2D::new(center.x, center.y);
                    drawing.push(TurtleDraw::DrawDot(center, *radius, *color));
                }
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let path = polygon.get_path();
                    let angle = Angle::degrees(*angle);
                    let xform = Transform2D::rotation(angle).then_translate([pos.x, pos.y].into());
                    let path = path.transform(&xform);
                    drawing.push(TurtleDraw::FillPolygon(fillcolor, pencolor, penwidth, path));
                }
                DrawCommand::Circle(points) => {
                    let (path, final_pos, final_angle) =
                        Self::circle_path(last_element, pct, points);
                    tpos = final_pos.into();
                    trot = final_angle;
                    drawing.push(TurtleDraw::DrawLines(pencolor, penwidth, path));
                }
                DrawCommand::SetPosition(pos) => {
                    tpos = [pos.x as f32, pos.y as f32];
                }
                DrawCommand::Text(pos, text) => {
                    let pos = Point::new(pos.x, pos.y);
                    drawing.push(TurtleDraw::DrawText(pos, text.to_string()));
                }
                DrawCommand::Filler | DrawCommand::Filled(_) => {}
                DrawCommand::StampTurtle
                | DrawCommand::Clear
                | DrawCommand::Reset
                | DrawCommand::BeginFill
                | DrawCommand::EndFill
                | DrawCommand::BeginPoly
                | DrawCommand::EndPoly => panic!("invalid draw command in gui"),
            }
        }

        if !cur_path.is_empty() {
            drawing.push(TurtleDraw::DrawLines(
                pencolor,
                penwidth,
                make_path(&mut cur_path),
            ));
        }

        if !turtle.hide_turtle {
            drawing.extend(Self::calculate_turtle(
                tpos,
                trot,
                fillcolor.into(),
                pencolor.into(),
                penwidth,
                turtle,
            ));
        }

        drawing
    }

    fn start_and_end(last_element: bool, pct: f32, line: &LineInfo) -> (Point, Point) {
        (
            [line.begin.x as f32, line.begin.y as f32].into(),
            if last_element {
                let end_x = line.begin.x as f32 + (line.end.x - line.begin.x) as f32 * pct;
                let end_y = line.begin.y as f32 + (line.end.y - line.begin.y) as f32 * pct;
                [end_x, end_y]
            } else {
                [line.end.x as f32, line.end.y as f32]
            }
            .into(),
        )
    }

    fn circle_path(
        last_element: bool,
        pct: f32,
        points: &[CirclePos],
    ) -> (Vec<LineSegment>, Point, f32) {
        let mut line_list = Vec::new();

        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut end: Point = Point::new(0., 0.);
        let mut angle = 0.;
        let (_, start_slice) = points[0].get_data();
        let mut start: Point = start_slice.into();

        let mut iter = points.windows(2).take(total + 1).peekable();
        while let Some(p) = iter.next() {
            let (end_angle, end_position_slice) = p[1].get_data();
            let end_position: Point = end_position_slice.into();
            let last_segment = iter.peek().is_none();
            end = end_position;
            if last_element && last_segment {
                let (_, begin) = p[0].get_data();
                let end_x = begin[0] + (end_position.x - begin[0]) * subpercent;
                let end_y = begin[1] + (end_position.y - begin[1]) * subpercent;
                end = [end_x, end_y].into();
            }
            if points[0].pen_down {
                line_list.push(LineSegment { start, end });
            }
            start = end_position;
            angle = end_angle;
        }

        (line_list, end, angle)
    }

    fn calculate_turtle<UI>(
        tpos: [f32; 2],
        trot: f32,
        fillcolor: TurtleColor,
        pencolor: TurtleColor,
        penwidth: f32,
        turtle: &IndividualTurtle<UI>,
    ) -> Vec<TurtleDraw> {
        let angle = Angle::degrees(trot);
        let transform = Transform2D::rotation(angle).then_translate(tpos.into());
        let mut result = Vec::new();

        for poly in &turtle.turtle_shape.poly {
            let path = poly.polygon.get_path();
            let path = path.transform(&transform);

            let fillcolor = fillcolor.color_or(&poly.fill);
            let pencolor = pencolor.color_or(&poly.outline);
            result.push(TurtleDraw::FillPolygon(fillcolor, pencolor, penwidth, path));
        }

        result
    }

    /*
    fn _circle_path(last_element: bool, pct: f32, points: &[CirclePos]) -> (Path, Point, f32) {
        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut tpos = Point::default();
        let mut trot = 0.;
        let path = Path::new(|b| {
            let (_, start) = points[0].get_data();

            b.move_to(start.into());

            let mut iter = points.windows(2).take(total + 1).peekable();
            while let Some(p) = iter.next() {
                let (end_angle, end) = p[1].get_data();
                let last_segment = iter.peek().is_none();
                tpos = end.into();
                if last_element && last_segment {
                    let (_, begin) = p[0].get_data();
                    let end_x = begin[0] + (end[0] - begin[0]) * subpercent;
                    let end_y = begin[1] + (end[1] - begin[1]) * subpercent;
                    tpos = [end_x, end_y].into();
                }
                if points[0].pen_down {
                    b.line_to(tpos);
                } else {
                    b.move_to(tpos);
                }
                trot = end_angle;
            }
        });
        (path, tpos, trot)
    }
    */
}

trait ConvertSimplePolygon {
    fn get_path(&self) -> Vec<LineSegment>;
}

impl ConvertSimplePolygon for PolygonPath {
    fn get_path(&self) -> Vec<LineSegment> {
        let mut path = Vec::new();
        let mut iter = self.path.iter();
        let mut start_pos = iter.next().unwrap();
        while let Some(end_pos) = iter.next() {
            let start = Point::new(start_pos[0], start_pos[1]);
            let end = Point::new(end_pos[0], end_pos[1]);
            path.push(LineSegment { start, end });
            start_pos = end_pos;
        }
        path
    }
}

trait Transformer {
    fn transform(&self, xform: &Transform2D<f32>) -> Vec<LineSegment>;
}

impl Transformer for Vec<LineSegment> {
    fn transform(&self, xform: &Transform2D<f32>) -> Vec<LineSegment> {
        self.iter()
            .map(|segment| segment.transform(xform))
            .collect()
    }
}
