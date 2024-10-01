use std::{
    collections::HashMap,
    io::Stdout,
    time::{Duration, Instant},
};

use ratatui::{backend::CrosstermBackend, crossterm::event, Frame, Terminal};

use crate::{
    generate::DrawCommand,
    polygon::TurtleShape,
    turtle::{task::TurtleTask, types::TurtleID, TurtleFlags},
};

use super::TurtleGui;

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

struct RatatuiInternal {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    last_id: TurtleID,
    turtle: HashMap<TurtleID, IndividualTurtle>,
}

impl RatatuiInternal {
    fn new() -> Self {
        let mut this = Self {
            terminal: ratatui::init(),
            last_id: TurtleID::default(),
            turtle: HashMap::new(),
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

fn draw(_frame: &mut Frame) {
    // TODO: draw stuff
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
            if let Err(e) = self.tui.terminal.draw(|frame| draw(frame)) {
                break Err(e);
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
                // self.on_tick();
                last_tick = Instant::now();
            }
        };
    }
}

impl TurtleGui for RatatuiFramework {
    fn new_turtle(&mut self) -> TurtleID {
        todo!()
    }

    fn shut_down(&mut self) {
        todo!();
    }

    fn clear_turtle(&mut self, turtle: TurtleID) {
        todo!()
    }

    fn set_shape(&mut self, turtle: TurtleID, shape: crate::polygon::TurtleShape) {
        todo!()
    }

    fn stamp(&mut self, turtle: TurtleID, pos: crate::ScreenPosition<f32>, angle: f32) -> usize {
        todo!()
    }

    fn clear_stamp(&mut self, turtle: TurtleID, stamp: usize) {
        todo!()
    }

    fn clear_stamps(&mut self, turtle: TurtleID, count: super::StampCount) {
        todo!()
    }

    fn get_turtle_shape_name(&mut self, turtle_id: TurtleID) -> String {
        todo!()
    }

    fn append_command(&mut self, turtle: TurtleID, cmd: crate::generate::DrawCommand) {
        todo!()
    }

    fn get_position(&self, turtle: TurtleID) -> usize {
        todo!()
    }

    fn fill_polygon(&mut self, turtle: TurtleID, cmd: crate::generate::DrawCommand, index: usize) {
        todo!()
    }

    fn undo(&mut self, turtle: TurtleID) {
        todo!()
    }

    fn pop(&mut self, turtle: TurtleID) -> Option<crate::generate::DrawCommand> {
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
        todo!()
    }
}
