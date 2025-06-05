use lyon_tessellation::{
    geom::euclid::{default::Point2D, default::Transform2D},
    math::Angle,
};

use crate::{polygon::PolygonPath, CirclePos, IndividualTurtle};
use crate::{DrawCommand, LineInfo, TurtleColor};

pub(crate) type Point = Point2D<f32>;

#[derive(Debug)]
pub enum TurtleDraw {
    DrawLines(TurtleColor, f32, Vec<LineSegment>),
    DrawDot(Point, f32, TurtleColor),
    DrawText(Point, String),
    FillPolygon(TurtleColor, TurtleColor, f32, Vec<LineSegment>),
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

// time   command                           command list
// 1       start fill @ cur pos             [Filler, Pos1]
// 2       move turtle to new position      [Filler, Pos1, Pos2]
// 3       move turtle to new position      [Filler, Pos1, Pos2, Pos3]
// 4       move turtle to new position      [Filler, Pos1, Pos2, Pos3, Pos4]
// 5       end fill                         [Polygon, Pos1, Pos2, Pos3, Pos4]

impl TurtleDraw {
    pub(crate) fn convert<UI>(pct: f32, turtle: &mut IndividualTurtle<UI>) {
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

        let mut iter = turtle.cmds.iter().skip(turtle.cvt.last_cmd_pos).peekable();
        let mut cur_path: Vec<(bool, Point)> = turtle.cvt.cur_path.clone();

        if let Some(pos) = turtle.cvt.last_fill_pos.take() {
            turtle.ops.truncate(pos);
        } else if let Some(pos) = turtle.cvt.last_ops_pos.take() {
            turtle.ops.truncate(pos);
        } else if let Some(pos) = turtle.cvt.poly_pos.take() {
            turtle.ops.truncate(pos);
        }

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            if !matches!(element, DrawCommand::Line(..))
                && !matches!(element, DrawCommand::SetHeading(..))
                && !cur_path.is_empty()
            {
                turtle.ops.push(TurtleDraw::DrawLines(
                    turtle.cvt.pencolor,
                    turtle.cvt.penwidth,
                    make_path(&mut cur_path),
                ));
            }

            match element {
                DrawCommand::Line(line) => {
                    let (start, end) = Self::start_and_end(last_element, pct, line);
                    turtle.cvt.tpos = [end.x, end.y];
                    if cur_path.is_empty() {
                        cur_path.push((line.pen_down, start));
                    }
                    cur_path.push((line.pen_down, end));
                }
                DrawCommand::SetPenColor(pc) => {
                    turtle.cvt.pencolor = *pc;
                }
                DrawCommand::SetPenWidth(pw) => turtle.cvt.penwidth = *pw,
                DrawCommand::SetFillColor(fc) => {
                    turtle.cvt.fillcolor = *fc;
                }
                DrawCommand::DrawPolygon(p) => {
                    turtle.ops.push(TurtleDraw::FillPolygon(
                        turtle.cvt.fillcolor,
                        turtle.cvt.pencolor,
                        turtle.cvt.penwidth,
                        p.get_path(),
                    ));
                }
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * pct
                    } else {
                        *end
                    };
                    turtle.cvt.trot = rotation;
                }
                DrawCommand::Dot(center, radius, color) => {
                    let center: Point = Point2D::new(center.x, center.y);
                    turtle
                        .ops
                        .push(TurtleDraw::DrawDot(center, *radius, *color));
                }
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let path = polygon.get_path();
                    let angle = Angle::degrees(*angle);
                    let xform = Transform2D::rotation(angle).then_translate([pos.x, pos.y].into());
                    let path = path.transform(&xform);
                    turtle.ops.push(TurtleDraw::FillPolygon(
                        turtle.cvt.fillcolor,
                        turtle.cvt.pencolor,
                        turtle.cvt.penwidth,
                        path,
                    ));
                }
                DrawCommand::Circle(points) => {
                    let (path, final_pos, final_angle) =
                        Self::circle_path(last_element, pct, points);
                    turtle.cvt.tpos = final_pos.into();
                    turtle.cvt.trot = final_angle;
                    turtle.ops.push(TurtleDraw::DrawLines(
                        turtle.cvt.pencolor,
                        turtle.cvt.penwidth,
                        path,
                    ));
                }
                DrawCommand::SetPosition(pos) => {
                    turtle.cvt.tpos = [pos.x as f32, pos.y as f32];
                }
                DrawCommand::Text(pos, text) => {
                    let pos = Point::new(pos.x, pos.y);
                    turtle.ops.push(TurtleDraw::DrawText(pos, text.to_string()));
                }
                DrawCommand::Filler => {
                    turtle.cvt.last_fill_pos = Some(turtle.ops.len());
                    println!("last fill pos = {}", turtle.ops.len());
                }
                DrawCommand::Filled(fill_point) => {
                    turtle.cvt.last_ops_pos = turtle.cvt.last_fill_pos.take();
                    turtle.cvt.last_fill_point = Some(*fill_point);
                }
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
            turtle.cvt.last_ops_pos = Some(turtle.ops.len());
            turtle.cvt.cur_path = cur_path.clone();
            turtle.ops.push(TurtleDraw::DrawLines(
                turtle.cvt.pencolor,
                turtle.cvt.penwidth,
                make_path(&mut cur_path),
            ));
        }

        if !turtle.hide_turtle {
            turtle.cvt.poly_pos = Some(turtle.ops.len());
            turtle.ops.extend(Self::calculate_turtle(turtle));
        }
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

    fn calculate_turtle<UI>(turtle: &IndividualTurtle<UI>) -> Vec<TurtleDraw> {
        let angle = Angle::degrees(turtle.cvt.trot);
        let transform = Transform2D::rotation(angle).then_translate(turtle.cvt.tpos.into());
        let mut result = Vec::new();

        for poly in &turtle.turtle_shape.poly {
            let path = poly.polygon.get_path();
            let path = path.transform(&transform);

            let fillcolor = turtle.cvt.fillcolor.color_or(&poly.fill);
            let pencolor = turtle.cvt.pencolor.color_or(&poly.outline);
            result.push(TurtleDraw::FillPolygon(
                fillcolor,
                pencolor,
                turtle.cvt.penwidth,
                path,
            ));
        }

        result
    }
}

trait ConvertSimplePolygon {
    fn get_path(&self) -> Vec<LineSegment>;
}

impl ConvertSimplePolygon for PolygonPath {
    fn get_path(&self) -> Vec<LineSegment> {
        let mut path = Vec::new();
        let mut iter = self.path.iter();
        let mut start_pos = iter.next().unwrap();
        for end_pos in iter {
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
