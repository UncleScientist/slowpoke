use lyon_tessellation::{
    geom::euclid::{default::Point2D, default::Transform2D},
    math::Angle,
};

use crate::{polygon::PolygonPath, turtle::handler::Progress, CirclePos, IndividualTurtle};
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

// forward in time (i.e. a new command was issued):
//  - turtle.cmds.len() increased, or
//  - pct increased
//
// backward in time (i.e. undo)
//  - turtle.cmds.len() decreased, or
//  - pct decreased
impl TurtleDraw {
    pub(crate) fn convert<UI>(fraction: f32, turtle: &mut IndividualTurtle<UI>) {
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

        println!(
            "============== command count = {}, fraction = {fraction}  ===================",
            turtle.cmds.len()
        );
        let cur_progress = Progress::of(turtle.cmds.len(), fraction);
        println!("last_progress = {:?}", turtle.cvt.last_progress);
        println!("cur_progress = {cur_progress:?}");

        if turtle.cmds.is_empty() {
            turtle.ops.clear();
            turtle.cvt.cur_path.clear();

            // TODO: don't have these two lines twice in this function
            turtle.cvt.set_trunc_pos(turtle.ops.len());
            turtle.ops.extend(Self::calculate_turtle(turtle));
            return;
        }

        let skip_to = cur_progress.cmd_index - 1;
        let mut iter = turtle.cmds.iter().skip(skip_to).peekable();
        let mut cur_path = turtle.cvt.cur_path.clone();
        turtle.cvt.cur_path = Vec::new();
        println!("  cur_path = {cur_path:?}");

        if let Some(pos) = turtle.cvt.get_trunc_pos() {
            turtle.ops.truncate(pos);
        }

        // are we backfilling a polygon?
        let refill_point = if let Some(DrawCommand::Filled(pos)) = iter.peek() {
            // TODO: don't recaulate the whole set of lines here -- just use what's already done
            iter = turtle.cmds.iter().skip(*pos).peekable();
            cur_path.clear();
            Some(*pos)
        } else {
            None
        };

        if !cur_path.is_empty() {
            if turtle.cvt.last_progress < cur_progress
                && turtle.cvt.last_progress.cmd_index == cur_progress.cmd_index
            {
                println!("popping last element of cur_path");
                cur_path.pop();
                /*
                if let Some(next_cmd) = iter.peek()
                    && _next_cmd.needs_time()
                {
                    println!("removing last non-line element {:?}", iter.peek());
                    cur_path.pop();
                }
                */
            } else if turtle.cvt.last_progress > cur_progress {
                // moving backward in time, to the previous cmd_index
                println!("moving backward in time");
                cur_path.pop();
                if let Some(next_cmd) = iter.peek()
                    && !next_cmd._needs_time()
                {
                    turtle.ops.pop();
                    let _x = iter.next();
                    dbg!(_x);
                    assert!(iter.next().is_none());
                    return;
                } else {
                    if !iter.peek().is_none() {
                        cur_path.pop();
                    }
                }
            }
        }

        while let Some(element) = iter.next() {
            println!("> drawing element {element:?}");
            let last_element = iter.peek().is_none() && fraction < 1.;
            if !matches!(element, DrawCommand::Line(..))
                && !matches!(element, DrawCommand::SetHeading(..))
                && !cur_path.is_empty()
            {
                let path = make_path(&mut cur_path);
                if !path.is_empty() {
                    turtle.ops.push(TurtleDraw::DrawLines(
                        turtle.cvt.pencolor,
                        turtle.cvt.penwidth,
                        path,
                    ));
                }
            }

            if last_element && fraction == 0.0 {
                continue;
            }

            match element {
                DrawCommand::Line(line) => {
                    let (start, end) = Self::start_and_end(last_element, fraction, line);
                    turtle.cvt.tpos = [end.x, end.y];
                    if cur_path.is_empty() {
                        cur_path.push((line.pen_down, start));
                    }
                    cur_path.push((line.pen_down, end));
                    // println!("extended cur_path: {cur_path:?}");
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
                        *start + (*end - *start) * fraction
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
                        Self::circle_path(last_element, fraction, points);
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
                    turtle.cvt.polygon_start_point = Some(turtle.ops.len());
                }
                DrawCommand::Filled(fill_point) => {
                    println!("** FILLED **");
                    turtle.cvt.last_fill_point = Some(*fill_point);
                    if let Some(prev) = refill_point
                        && prev == *fill_point
                    {
                        // do nothing
                    } else {
                        turtle.cvt.set_trunc_pos(*fill_point);
                    }
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
            turtle.cvt.set_trunc_pos(turtle.ops.len());
            turtle.cvt.cur_path = cur_path.clone();
            let path = make_path(&mut cur_path);
            if !path.is_empty() {
                turtle.ops.push(TurtleDraw::DrawLines(
                    turtle.cvt.pencolor,
                    turtle.cvt.penwidth,
                    path,
                ));
            }
        }

        if !turtle.hide_turtle {
            turtle.cvt.set_trunc_pos(turtle.ops.len());
            turtle.ops.extend(Self::calculate_turtle(turtle));
        }

        if let Some(pos) = turtle.cvt.polygon_start_point.take() {
            turtle.cvt.set_trunc_pos(pos);
        }

        turtle
            .cvt
            .last_progress
            .set_progress(turtle.cmds.len(), fraction);
        println!("leaving last_cmd_pos = {:?}", turtle.cvt.last_progress);
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::ScreenPosition;

    fn get_turtle() -> IndividualTurtle<usize> {
        IndividualTurtle::<usize>::default()
    }

    #[test]
    fn test_create_turtle() {
        let turtle = get_turtle();

        assert_eq!(turtle.ops.len(), 0);
    }

    #[test]
    fn test_draw_line() {
        let mut turtle = get_turtle();

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::default(),
            end: ScreenPosition::default(),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
    }

    #[test]
    fn test_polygon() {
        let mut turtle = get_turtle();

        println!("-- filler --");
        let index = 0;
        turtle.cmds.push(DrawCommand::Filler);
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 1);
        assert_eq!(turtle.cvt.cur_path.len(), 0);

        println!("-- 1st line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 0),
            end: ScreenPosition::new(10, 0),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        println!("-- 2nd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(10, 0),
            end: ScreenPosition::new(0, 10),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 3);

        println!("-- 3rd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 10),
            end: ScreenPosition::new(0, 0),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 4);

        println!("-- polygon --");
        let polygon = PolygonPath::new(&[[0., 0.], [10., 0.], [0., 10.]]);
        turtle.cmds[index] = DrawCommand::DrawPolygon(polygon);
        turtle.cmds.push(DrawCommand::Filled(index));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 3);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 0);
    }

    #[test]
    fn test_two_polygons() {
        let mut turtle = get_turtle();

        println!("-- filler --");
        let index = turtle.ops.len();
        turtle.cmds.push(DrawCommand::Filler);
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 1);

        println!("-- 1st line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 0),
            end: ScreenPosition::new(10, 0),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);

        println!("-- 2nd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(10, 0),
            end: ScreenPosition::new(0, 10),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);

        println!("-- 3rd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 10),
            end: ScreenPosition::new(0, 0),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);

        println!("-- polygon --");
        let polygon = PolygonPath::new(&[[0., 0.], [10., 0.], [0., 10.]]);
        turtle.cmds[index] = DrawCommand::DrawPolygon(polygon);
        turtle.cmds.push(DrawCommand::Filled(index));
        TurtleDraw::convert(1., &mut turtle);
        dbg!(&turtle.ops);
        assert_eq!(turtle.ops.len(), 3);

        println!("-- go to new location --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 0),
            end: ScreenPosition::new(100, 100),
            pen_down: false,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        println!("-- 2nd filler --");
        let index = turtle.ops.len();
        turtle.cmds.push(DrawCommand::Filler);
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 3);

        println!("-- 2nd 1st line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(100, 100),
            end: ScreenPosition::new(110, 100),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 4);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        println!("-- 2nd 2nd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(110, 100),
            end: ScreenPosition::new(100, 110),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 4);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 3);

        println!("-- 2nd 3rd line --");
        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(100, 110),
            end: ScreenPosition::new(100, 100),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 4);
        assert_eq!(turtle.cvt.cur_path.len(), 4);

        println!("-- 2nd polygon --");
        let polygon = PolygonPath::new(&[[100., 100.], [110., 100.], [100., 110.]]);
        turtle.cmds[index] = DrawCommand::DrawPolygon(polygon);
        turtle.cmds.push(DrawCommand::Filled(index));
        TurtleDraw::convert(1., &mut turtle);
        dbg!(&turtle.ops);
        assert_eq!(turtle.ops.len(), 5);
    }

    #[test]
    fn test_undo() {
        let mut turtle = get_turtle();

        // TurtleDraw::convert(0.0, &mut turtle);
        assert_eq!(turtle.ops.len(), 0);
        assert_eq!(turtle.cvt.cur_path.len(), 0);

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 0),
            end: ScreenPosition::new(10, 10),
            pen_down: true,
        }));

        TurtleDraw::convert(0.0, &mut turtle);
        assert_eq!(turtle.ops.len(), 1);
        assert_eq!(turtle.cvt.cur_path.len(), 0);

        TurtleDraw::convert(0.25, &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        TurtleDraw::convert(0.5, &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        TurtleDraw::convert(1.0, &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        TurtleDraw::convert(0.5, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        println!("POP");
        turtle.cmds.pop();
        TurtleDraw::convert(1.0, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.ops.len(), 1);
        assert_eq!(turtle.cvt.cur_path.len(), 0);
    }

    #[test]
    fn test_two_segment_undo() {
        let mut turtle = get_turtle();

        TurtleDraw::convert(0.0, &mut turtle);
        assert_eq!(turtle.ops.len(), 1);
        assert_eq!(turtle.cvt.cur_path.len(), 0);

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(0, 0),
            end: ScreenPosition::new(10, 10),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(10, 10),
            end: ScreenPosition::new(42, 81),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        assert_eq!(turtle.ops.len(), 2);
        assert_eq!(turtle.cvt.cur_path.len(), 3);

        turtle
            .cmds
            .push(DrawCommand::Dot(Point::new(42., 81.), 2.3, "black".into()));
        TurtleDraw::convert(1.0, &mut turtle);
        assert_eq!(turtle.ops.len(), 3);

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(41, 81),
            end: ScreenPosition::new(100, 0),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);
        assert_eq!(turtle.ops.len(), 4);

        turtle.cmds.push(DrawCommand::Line(LineInfo {
            begin: ScreenPosition::new(100, 0),
            end: ScreenPosition::new(-5, -12),
            pen_down: true,
        }));
        TurtleDraw::convert(1., &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 3);
        assert_eq!(turtle.ops.len(), 4);

        TurtleDraw::convert(0.5, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 3);

        TurtleDraw::convert(0.0, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);

        turtle.cmds.pop();
        TurtleDraw::convert(1.0, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 2);
        assert_eq!(turtle.ops.len(), 4);

        turtle.cmds.pop();
        TurtleDraw::convert(1.0, &mut turtle);
        dbg!(&turtle.cvt.cur_path);
        assert_eq!(turtle.cvt.cur_path.len(), 0);
        assert_eq!(turtle.ops.len(), 3);

        assert!(false);
    }
}
