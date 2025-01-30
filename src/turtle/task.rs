use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    color_names::TurtleColor,
    command::{Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd},
    comms::{Request, Response},
    gui::{events::TurtleEvent, Progression, StampCount, TurtleGui},
    polygon::{generate_default_shapes, ShapeComponent, TurtleShape},
    ScreenPosition, Shape, Turtle, TurtleShapeName,
};

use super::{types::TurtleThread, TurtleCommand, TurtleData, TurtleFlags, TurtleID, TurtleTimer};

use crate::generate::TurtlePosition;

#[derive(Default)]
pub(crate) struct TurtleTask {
    issue_command: Option<Sender<Request>>,
    receive_command: Option<Receiver<Request>>,
    turtle_list: Vec<TurtleData>,
    shapes: HashMap<String, TurtleShape>,
    winsize: [isize; 2],
    exit_on_click: bool,
}

macro_rules! spawn {
    ($task:expr, $td:expr, $idx:expr, $func:expr, $($args:tt)*) => {
        {
            let _turtle :TurtleID = $idx.into();
            let _thread = $td.next_thread.get();

            let mut _new_turtle = $td.spawn(_thread,
                $task.issue_command.as_ref().unwrap().clone());

            let _ = std::thread::spawn(move || {
                $func(&mut _new_turtle, $($args)*);
            });
        }
    };
}

#[derive(PartialEq)]
pub(crate) enum EventResult {
    Continue,
    ShutDown,
}

impl TurtleTask {
    pub(crate) fn new(flags: &mut TurtleFlags) -> Self {
        let issue_command = flags.issue_command.take();
        let receive_command = flags.receive_command.take();
        Self {
            issue_command,
            receive_command,
            turtle_list: vec![TurtleData::new()],
            shapes: generate_default_shapes(),
            ..Self::default()
        }
    }

    pub(crate) fn progress(&self, tid: TurtleID) -> (f32, Progression) {
        (
            self.turtle_list[tid].state.percent,
            self.turtle_list[tid].state.progression,
        )
    }

    pub(crate) fn popup_result(
        &mut self,
        turtle: TurtleID,
        thread: TurtleThread,
        response: Response,
    ) {
        let _ = self.turtle_list[turtle].responder[&thread].send(response);
    }

    pub(crate) fn popup_cancelled(&mut self, turtle: TurtleID, thread: TurtleThread) {
        let _ = self.turtle_list[turtle].responder[&thread].send(Response::Cancel);
    }

    pub(crate) fn handle_event(
        &mut self,
        turtle: Option<TurtleID>,
        thread: Option<TurtleThread>,
        event: &TurtleEvent,
    ) -> EventResult {
        match event {
            TurtleEvent::WindowResize(width, height) => {
                self.winsize = [*width, *height];
                if turtle.is_none() {
                    assert!(thread.is_none());
                } else {
                    let turtle = turtle.expect("missing turtle from window resize");
                    let thread = thread.expect("missing thread from window resize");
                    let _ = self.turtle_list[turtle].responder[&thread].send(Response::Done);
                }
            }
            TurtleEvent::KeyPress(ch) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onkeypress.get(ch).copied() {
                        if !turtle.pending_key_event() {
                            let ch = *ch;
                            spawn!(self, turtle, idx, func, ch);
                        }
                    }
                }
            }
            TurtleEvent::KeyRelease(ch) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onkeyrelease.get(ch).copied() {
                        if !turtle.pending_key_event() {
                            let ch = *ch;
                            spawn!(self, turtle, idx, func, ch);
                        }
                    }
                }
            }
            TurtleEvent::MousePress(x, y) => {
                if self.exit_on_click {
                    return EventResult::ShutDown;
                }
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmousepress {
                        let (x, y) = (*x, *y);
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            TurtleEvent::MouseRelease(x, y) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmouserelease {
                        let (x, y) = (*x, *y);
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            #[cfg(feature = "iced")]
            TurtleEvent::MousePosition(_, _) => unreachable!(),
            TurtleEvent::MouseDrag(x, y) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmousedrag {
                        let (x, y) = (*x, *y);
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            TurtleEvent::_Timer => todo!(),
            TurtleEvent::Unhandled => {}
        }

        EventResult::Continue
    }

    pub(crate) fn run_turtle<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let turtle = TurtleID::new(0);
        let thread = TurtleThread::new(0);
        let issue_command = self.issue_command.as_ref().unwrap().clone();
        let mut primary = self.turtle_list[turtle].spawn(thread, issue_command);
        let _ = std::thread::spawn(move || func(&mut primary));
    }

    pub(crate) fn tick<G: TurtleGui>(&mut self, gui: &mut G) {
        while let Ok(req) = self.receive_command.as_ref().unwrap().try_recv() {
            self.handle_command(req, gui);
        }

        for turtle in &mut self.turtle_list {
            let timer = match &turtle.event.ontimer {
                Some(timer) if timer.prev.elapsed() > timer.time => {
                    Some((timer.prev.elapsed(), timer.func))
                }
                _ => None,
            };

            if let Some((duration, func)) = timer {
                spawn!(self, turtle, turtle.turtle_id, func, duration);
                turtle.event.ontimer = None;
            }

            turtle.time_passes(gui, 0.01); // TODO: use actual time delta
        }
    }

    pub(crate) fn hatch_turtle<G: TurtleGui>(&mut self, gui: &mut G) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        let turtle = gui.new_turtle();
        let thread = TurtleThread::new(0);

        let mut td = TurtleData::new();
        td.responder.insert(thread, finished);
        td.turtle_id = turtle;
        self.turtle_list.push(td);

        Turtle::init(
            self.issue_command.as_ref().unwrap().clone(),
            command_complete,
            turtle,
            thread,
        )
    }

    fn screen_cmd<G: TurtleGui>(
        &mut self,
        turtle: TurtleID,
        cmd: ScreenCmd,
        thread: TurtleThread,
        gui: &mut G,
    ) {
        let resp = self.turtle_list[turtle]
            .responder
            .get(&thread)
            .unwrap()
            .clone();
        match cmd {
            ScreenCmd::Bye => {
                gui.shut_down();
            }
            ScreenCmd::ExitOnClick => {
                // Note: this does not send back a response as it is meant to just
                // block until the user clicks the mouse.
                self.exit_on_click = true;
            }
            ScreenCmd::SetTitle(s) => {
                gui.set_title(s);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BgPic(_picdata) => todo!(),
            ScreenCmd::RegisterShape(name, shape) => {
                match shape {
                    Shape::Polygon(ShapeComponent { polygon, .. }) => {
                        self.shapes
                            .insert(name.clone(), TurtleShape::new(&name, polygon));
                    }
                    Shape::Image(_) => todo!(),
                    Shape::Compound(s) => {
                        self.shapes
                            .insert(name.clone(), TurtleShape::multi(&name, &s));
                    }
                };
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::SetSize(s) => {
                // Note: we don't send "done" here because we need to
                // wait for the resize event from the GUI
                gui.resize(turtle, thread, s[0], s[1]);
            }
            ScreenCmd::ShowTurtle(t) => {
                gui.set_visible(turtle, t);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Speed(s) => {
                self.turtle_list[turtle].state.speed = s;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                gui.bgcolor([r, g, b, 1.].into());
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                gui.bgcolor("white".into());
                gui.clearscreen();
                self.turtle_list.truncate(1);
                self.turtle_list[0].reset();
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                gui.clear_stamp(turtle, id);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamps(count) => {
                #[allow(clippy::comparison_chain)]
                if count < 0 {
                    gui.clear_stamps(turtle, StampCount::Reverse(count.unsigned_abs()));
                } else if count == 0 {
                    gui.clear_stamps(turtle, StampCount::All);
                } else {
                    gui.clear_stamps(turtle, StampCount::Forward(count.unsigned_abs()));
                }
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn input_cmd(&mut self, turtle: TurtleID, cmd: InputCmd, thread: TurtleThread) {
        let resp = self.turtle_list[turtle]
            .responder
            .get(&thread)
            .unwrap()
            .clone();
        match cmd {
            InputCmd::Timer(f, d) => {
                let _ = self.turtle_list[turtle]
                    .event
                    .ontimer
                    .insert(TurtleTimer::new(f, d));
                let _ = resp.send(Response::Done);
            }
            InputCmd::KeyRelease(f, k) => {
                self.turtle_list[turtle].event.onkeyrelease.insert(k, f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::KeyPress(f, k) => {
                self.turtle_list[turtle].event.onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MouseDrag(f) => {
                self.turtle_list[turtle].event.onmousedrag = Some(f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MousePress(f) => {
                self.turtle_list[turtle].event.onmousepress = Some(f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MouseRelease(f) => {
                self.turtle_list[turtle].event.onmouserelease = Some(f);
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn data_cmd<G: TurtleGui>(
        &mut self,
        turtle: TurtleID,
        cmd: &DataCmd,
        thread: TurtleThread,
        gui: &mut G,
    ) {
        let resp = self.turtle_list[turtle]
            .responder
            .get(&thread)
            .unwrap()
            .clone();

        let _ = match cmd {
            DataCmd::GetTurtles => {
                let mut turtles = Vec::new();
                for turtle in &mut self.turtle_list {
                    let thread = turtle.next_thread.get();
                    let thing = turtle.spawn(thread, self.issue_command.as_ref().unwrap().clone());
                    turtles.push(thing);
                }
                resp.send(Response::Turtles(turtles))
            }
            DataCmd::GetShapes => {
                resp.send(Response::ShapeList(self.shapes.keys().cloned().collect()))
            }
            DataCmd::GetFillingState => resp.send(Response::IsFilling(
                self.turtle_list[turtle].state.insert_fill.is_some(),
            )),
            DataCmd::GetPenState => resp.send(Response::IsPenDown(
                self.turtle_list[turtle].state.turtle.get_pen_state(),
            )),
            DataCmd::GetScreenSize => resp.send(Response::ScreenSize(self.winsize)),
            DataCmd::Visibility => resp.send(Response::Visibility(gui.is_visible(turtle))),
            DataCmd::GetPoly => resp.send(Response::Polygon(
                self.turtle_list[turtle].state.shape_poly.verticies.clone(),
            )),
            DataCmd::TurtleShape(shape) => {
                if let TurtleShapeName::Shape(name) = shape {
                    if let Some(shape) = self.shapes.get(name) {
                        gui.set_shape(turtle, shape.clone());
                    }
                }
                resp.send(Response::Name(gui.get_turtle_shape_name(turtle)))
            }
            DataCmd::UndoBufferEntries => resp.send(Response::Count(gui.undo_count(turtle))),
            DataCmd::Towards(xpos, ypos) => {
                let curpos: ScreenPosition<f32> = self.turtle_list[turtle].state.turtle.pos();
                let x = xpos - curpos.x;
                let y = ypos + curpos.y;

                let heading = self.turtle_list[turtle]
                    .state
                    .turtle
                    .radians_to_turtle(y.atan2(x));

                resp.send(Response::Heading(heading))
            }
            DataCmd::Position => resp.send(Response::Position(
                self.turtle_list[turtle].state.turtle.pos(),
            )),
            DataCmd::Heading => {
                let angle = self.turtle_list[turtle].state.turtle.angle();
                let angle = self.turtle_list[turtle]
                    .state
                    .turtle
                    .degrees_to_turtle(angle);
                resp.send(Response::Heading(angle))
            }
            DataCmd::Stamp => {
                self.turtle_list[turtle].queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Stamp),
                    turtle,
                    thread,
                });
                Ok(())
            }
            DataCmd::NumInput(title, prompt) => {
                gui.numinput(turtle, thread, title, prompt);
                Ok(())
            }
            DataCmd::TextInput(title, prompt) => {
                gui.textinput(turtle, thread, title, prompt);
                Ok(())
            }
        };
    }

    fn draw_cmd(&mut self, turtle: TurtleID, cmd: DrawRequest, thread: TurtleThread) {
        let is_stamp = cmd.is_stamp();
        self.turtle_list[turtle].queue.push_back(TurtleCommand {
            cmd,
            turtle,
            thread,
        });

        // FIXME: data commands (Command::Data(_)) require all queued entries to be
        // processed before sending a response, even if `respond_immediately` is set
        if self.turtle_list[turtle].state.respond_immediately {
            self.turtle_list[turtle].send_response(thread, is_stamp);
        }
    }

    fn handle_command<G: TurtleGui>(&mut self, req: Request, gui: &mut G) {
        let turtle = req.turtle;
        let thread = req.thread;

        match req.cmd {
            Command::ShutDown => {
                let tid = self.turtle_list[turtle].responder.remove(&thread);
                self.turtle_list[turtle].event.pending_keys = false;
                assert!(tid.is_some());
            }
            Command::Screen(cmd) => self.screen_cmd(turtle, cmd, thread, gui),
            Command::Draw(cmd) => self.draw_cmd(turtle, cmd, thread),
            Command::Input(cmd) => self.input_cmd(turtle, cmd, thread),
            Command::Data(cmd) => self.data_cmd(turtle, &cmd, thread, gui),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle(gui);
                let resp = &self.turtle_list[turtle].responder[&thread];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }
}
