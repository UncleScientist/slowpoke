use std::{
    cell::RefCell,
    collections::HashMap,
    io::Stdout,
    time::{Duration, Instant},
};

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
    generate::DrawCommand,
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
    fn draw(&self, ctx: &mut Context) {
        let mut pencolor = Color::White;
        let mut trot = 0f32;
        let mut iter = self.cmds.iter().peekable();
        let mut pct = 1.;

        while let Some(cmd) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;
            match cmd {
                DrawCommand::Line(l) => {
                    if l.pen_down {
                        ctx.draw(&Line::new(
                            l.begin.x as f64,
                            l.begin.y as f64,
                            l.end.x as f64,
                            l.end.y as f64,
                            pencolor,
                        ));
                    }
                }
                DrawCommand::Clear => todo!(),
                DrawCommand::Reset => todo!(),
                DrawCommand::Filler => todo!(),
                DrawCommand::Filled(_) => todo!(),
                DrawCommand::BeginFill => todo!(),
                DrawCommand::EndFill => todo!(),
                DrawCommand::BeginPoly => todo!(),
                DrawCommand::EndPoly => todo!(),
                DrawCommand::StampTurtle => todo!(),
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
                DrawCommand::Circle(_) => todo!(),
                DrawCommand::Text(_, _) => todo!(),
            }
        }
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
            .block(Block::bordered().title(self.tui.title.clone()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                for turtle in self.tui.turtle.values() {
                    turtle.draw(ctx);
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
        todo!();
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
        todo!()
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
