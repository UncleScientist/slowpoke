use std::{
    cell::RefCell,
    collections::HashMap,
    io::Stdout,
    time::{Duration, Instant},
};

use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::event,
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Context, Line},
        Block,
    },
    Frame, Terminal,
};

use crate::{
    color_names::TurtleColor,
    generate::{CirclePos, DrawCommand},
    polygon::TurtleShape,
    turtle::{task::TurtleTask, types::TurtleID, TurtleFlags},
};

use super::{StampCount, TurtleGui};

pub(crate) struct RatatuiFramework {
    tt: TurtleTask,
    tui: RatatuiInternal,
}

#[derive(Default)]
struct IndividualTurtle {
    cmds: Vec<DrawCommand>,
    drawing: Vec<RatatuiDrawCmd>,
    has_new_cmd: bool,
    turtle_shape: TurtleShape,
    hide_turtle: bool,
}

impl IndividualTurtle {
    fn draw(&self, ctx: &mut Context, pct: f32) {
        let mut pencolor = Color::Black;
        let mut trot = 0f32;
        let mut tpos = [0f64, 0f64];
        let mut iter = self.cmds.iter().peekable();

        while let Some(cmd) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            match cmd {
                DrawCommand::Line(l) => {
                    let (begin_x, begin_y): (f64, f64) = (l.begin.x as f64, l.begin.y as f64);
                    let (end_x, end_y) = if last_element {
                        let end_x = l.begin.x as f32 + (l.end.x - l.begin.x) as f32 * pct;
                        let end_y = l.begin.y as f32 + (l.end.y - l.begin.y) as f32 * pct;
                        tpos = [end_x as f64, end_y as f64];
                        (tpos[0], tpos[1])
                    } else {
                        tpos = [l.end.x as f64, l.end.y as f64];
                        (tpos[0], tpos[1])
                    };
                    if l.pen_down {
                        ctx.draw(&Line::new(begin_x, begin_y, end_x, end_y, pencolor));
                    }
                }
                DrawCommand::Filler => todo!(),
                DrawCommand::Filled(_) => todo!(),
                DrawCommand::SetPenColor(pc) => pencolor = pc.into(),
                DrawCommand::SetPenWidth(_) => {
                    // TODO: figure out pen width for ratatui
                }
                DrawCommand::SetFillColor(_) => {
                    // TODO: figure out fill color for ratatui
                }
                DrawCommand::SetPosition(_) => todo!(),
                DrawCommand::DrawPolygon(_) => todo!(),
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * pct
                    } else {
                        *end
                    };
                    trot = rotation;
                }
                DrawCommand::DrawDot(_, _, _) => todo!(),
                DrawCommand::DrawPolyAt(_, _, _) => todo!(),
                DrawCommand::Circle(points) => {
                    let (line_list, final_pos, final_angle) =
                        Self::circle_path(last_element, pct, points);
                    tpos = [final_pos[0] as f64, final_pos[1] as f64];
                    trot = final_angle;
                    for line in line_list {
                        ctx.draw(&Line::new(
                            line.0[0] as f64,
                            line.0[1] as f64,
                            line.1[0] as f64,
                            line.1[1] as f64,
                            pencolor,
                        ));
                    }
                }
                DrawCommand::Text(_, _) => todo!(),
                DrawCommand::StampTurtle
                | DrawCommand::Clear
                | DrawCommand::Reset
                | DrawCommand::BeginFill
                | DrawCommand::EndFill
                | DrawCommand::BeginPoly
                | DrawCommand::EndPoly => panic!("invalid draw command in gui"),
            }
        }

        if !self.hide_turtle {
            let angle = Angle::degrees(trot);
            let tpos = [tpos[0] as f32, tpos[1] as f32];
            let transform = Transform2D::rotation(angle).then_translate(tpos.into());
            for poly in &self.turtle_shape.poly {
                for pair in poly.polygon.path.as_slice().windows(2) {
                    let p1 = pair[0];
                    let p2 = pair[1];
                    let start = transform.transform_point(p1.into());
                    let end = transform.transform_point(p2.into());

                    let pencolor = if matches!(poly.outline, TurtleColor::CurrentColor) {
                        pencolor
                    } else {
                        (&poly.outline).into()
                    };

                    ctx.draw(&Line::new(
                        start.x as f64,
                        start.y as f64,
                        end.x as f64,
                        end.y as f64,
                        pencolor,
                    ));
                }
            }
        }
    }

    // returns path, final point, and final angle
    fn circle_path(
        last_element: bool,
        pct: f32,
        points: &[CirclePos],
    ) -> (Vec<([f32; 2], [f32; 2])>, [f32; 2], f32) {
        let mut line_list = Vec::new();

        let (total, subpercent) = if last_element {
            let partial = (points.len() - 1) as f32 * pct;
            let p = (partial.floor() as i64).checked_abs().expect("too small") as usize;
            (p, (partial - partial.floor()))
        } else {
            (points.len() - 1, 1_f32)
        };
        let mut tpos = [0., 0.];
        let mut trot = 0.;
        let (_, mut start) = points[0].get_data();

        let mut iter = points.windows(2).take(total + 1).peekable();
        while let Some(p) = iter.next() {
            let (end_angle, end) = p[1].get_data();
            let last_segment = iter.peek().is_none();
            tpos = end;
            if last_element && last_segment {
                let (_, begin) = p[0].get_data();
                let end_x = begin[0] + (end[0] - begin[0]) * subpercent;
                let end_y = begin[1] + (end[1] - begin[1]) * subpercent;
                tpos = [end_x, end_y].into();
            }
            if points[0].pen_down {
                line_list.push((start, tpos));
            }
            start = end;
            trot = end_angle;
        }
        (line_list, tpos, trot)
    }
}

struct RatatuiInternal {
    terminal: RefCell<Terminal<CrosstermBackend<Stdout>>>,
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
    title: String, // TODO: implement popups
}

impl RatatuiInternal {
    fn new() -> Self {
        let mut this = Self {
            terminal: RefCell::new(ratatui::init()),
            last_id: TurtleID::default(),
            turtle: HashMap::new(),
            title: "*default title*".to_string(),
        };
        let _turtle = this.new_turtle();
        this
    }

    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(
            id,
            IndividualTurtle {
                has_new_cmd: true,
                ..Default::default()
            },
        );
        id
    }
}

enum RatatuiDrawCmd {
    Line,
    Text,
}

impl RatatuiFramework {
    pub(crate) fn start(mut flags: TurtleFlags) {
        let func = flags.start_func.take();

        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let tui = RatatuiInternal::new();
        let mut rata = Self { tt, tui };
        rata.run();

        ratatui::restore();
    }

    fn run(&mut self) {
        let tick_rate = Duration::from_millis(1000 / 60);
        let mut last_tick = Instant::now();
        let result = loop {
            {
                let mut term = self.tui.terminal.borrow_mut();
                if let Err(e) = term.draw(|frame| self.draw(frame)) {
                    break Err(e);
                }
            }
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            match event::poll(timeout) {
                Err(e) => break Err(e),
                Ok(true) => {
                    let event = event::read().expect("should not have failed");
                    break Ok(event);
                }
                Ok(false) => {}
            }

            if last_tick.elapsed() >= tick_rate {
                self.tt.tick(&mut self.tui);
                last_tick = Instant::now();
            }
        };
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let widget = Canvas::default()
            .background_color(Color::White)
            .block(Block::bordered().title(self.tui.title.clone()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                for (tid, turtle) in self.tui.turtle.iter() {
                    let (pct, _) = self.tt.progress(*tid);
                    turtle.draw(ctx, pct);
                }
            })
            .x_bounds([-200., 200.])
            .y_bounds([-200., 200.]);

        frame.render_widget(widget, area);
    }
}

impl TurtleGui for RatatuiInternal {
    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(
            id,
            IndividualTurtle {
                has_new_cmd: true,
                ..Default::default()
            },
        );
        id
    }

    fn shut_down(&mut self) {
        ratatui::restore();
        std::process::exit(0);
    }

    fn clear_turtle(&mut self, turtle: TurtleID) {
        todo!()
    }

    fn set_shape(&mut self, turtle: TurtleID, shape: crate::polygon::TurtleShape) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape = shape;
        turtle.has_new_cmd = true;
    }

    fn stamp(&mut self, turtle: TurtleID, pos: crate::ScreenPosition<f32>, angle: f32) -> usize {
        todo!()
    }

    fn clear_stamp(&mut self, turtle: TurtleID, stamp: usize) {
        todo!()
    }

    fn clear_stamps(&mut self, turtle: TurtleID, count: StampCount) {
        todo!()
    }

    fn get_turtle_shape_name(&mut self, turtle: TurtleID) -> String {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape.name.clone()
    }

    fn append_command(&mut self, turtle: TurtleID, cmd: DrawCommand) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(cmd);
    }

    fn get_position(&self, turtle: TurtleID) -> usize {
        todo!()
    }

    fn fill_polygon(&mut self, turtle: TurtleID, cmd: DrawCommand, index: usize) {
        todo!()
    }

    fn undo(&mut self, turtle: TurtleID) {
        todo!()
    }

    fn pop(&mut self, turtle: TurtleID) -> Option<DrawCommand> {
        todo!()
    }

    fn undo_count(&self, turtle: TurtleID) -> usize {
        todo!()
    }

    fn numinput(
        &mut self,
        turtle: TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        todo!()
    }

    fn textinput(
        &mut self,
        turtle: TurtleID,
        thread: crate::turtle::types::TurtleThread,
        title: &str,
        prompt: &str,
    ) {
        todo!()
    }

    fn bgcolor(&mut self, color: crate::color_names::TurtleColor) {
        // TODO: change background color
    }

    fn resize(
        &mut self,
        turtle: TurtleID,
        thread: crate::turtle::types::TurtleThread,
        width: isize,
        height: isize,
    ) {
        todo!()
    }

    fn set_visible(&mut self, turtle: TurtleID, visible: bool) {
        todo!()
    }

    fn is_visible(&self, turtle: TurtleID) -> bool {
        todo!()
    }

    fn clearscreen(&mut self) {
        todo!()
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
//
//TODO: use tryfrom instead?
impl From<&TurtleColor> for Color {
    fn from(value: &TurtleColor) -> Self {
        if let TurtleColor::Color(r, g, b) = value {
            Color::Rgb((*r * 255.) as u8, (*g * 255.) as u8, (*b * 255.) as u8)
        } else {
            todo!()
        }
    }
}
