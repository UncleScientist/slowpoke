pub(crate) mod handler;
pub(crate) mod task;
pub(crate) mod types;

use types::{TurtleID, TurtleThread};

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    time::{Duration, Instant},
};

use crate::{
    command::{
        Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd, TimedDrawCmd,
    },
    comms::{Request, Response},
    generate::{CurrentTurtleState, DrawCommand, TurtlePosition},
    gui::{Progression, TurtleGui},
    polygon::PolygonPath,
    speed::Speed,
    ScreenPosition,
};

#[derive(Debug)]
struct TurtleCommand {
    cmd: DrawRequest,
    turtle: TurtleID,
    thread: TurtleThread,
}

// T == user interface code
pub struct SlowpokeLib<T> {
    pub(crate) size: [isize; 2],
    pub(crate) title: String,
    data: PhantomData<T>,
}

impl<T> Default for SlowpokeLib<T> {
    fn default() -> Self {
        Self {
            size: [800, 800],
            title: "Turtle".to_string(),
            data: PhantomData,
        }
    }
}

impl<T: std::fmt::Debug + TurtleUserInterface> SlowpokeLib<T> {
    #[must_use]
    pub fn new() -> SlowpokeLib<T> {
        SlowpokeLib::default()
    }

    #[must_use]
    pub fn with_size(mut self, x: isize, y: isize) -> Self {
        self.size = [x, y];
        self
    }

    #[must_use]
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn run<F: FnOnce(&mut Turtle) + Send + 'static>(&self, func: F) {
        Turtle::run(self, func);
    }
}

#[derive(Debug)]
pub struct Turtle {
    issue_command: Sender<Request>,
    command_complete: Receiver<Response>,
    turtle: TurtleID,
    thread: TurtleThread,
    tracer: RefCell<bool>,
    // data: PhantomData<T>,
}

impl Drop for Turtle {
    fn drop(&mut self) {
        let _ = self
            .issue_command
            .send(Request::shut_down(self.turtle, self.thread));
    }
}

pub trait TurtleUserInterface {
    fn start(flags: TurtleFlags);
}

impl Turtle {
    pub fn run<T: TurtleUserInterface, F: FnOnce(&mut Turtle) + Send + 'static>(
        args: &SlowpokeLib<T>,
        func: F,
    ) {
        let xsize = to_f32(args.size[0]);
        let ysize = to_f32(args.size[1]);

        let (issue_command, receive_command) = mpsc::channel();

        let flags = TurtleFlags {
            start_func: Some(Box::new(func)),
            issue_command: Some(issue_command),
            receive_command: Some(receive_command),
            title: args.title.clone(),
            size: [xsize, ysize],
        };

        T::start(flags);
    }

    pub(crate) fn init(
        issue_command: Sender<Request>,
        command_complete: Receiver<Response>,
        turtle: TurtleID,
        thread: TurtleThread,
    ) -> Self {
        Self {
            issue_command,
            command_complete,
            turtle,
            thread,
            tracer: true.into(),
        }
    }

    pub(crate) fn do_draw(&mut self, cmd: DrawRequest) {
        let _ = self.do_command(Command::Draw(cmd));
    }

    pub(crate) fn do_screen(&mut self, cmd: ScreenCmd) {
        let _ = self.do_command(Command::Screen(cmd));
    }

    pub(crate) fn do_input(&self, cmd: InputCmd) {
        let _ = self.do_command(Command::Input(cmd));
    }

    pub(crate) fn do_data(&self, cmd: DataCmd) -> Response {
        self.do_command(Command::Data(cmd))
    }

    pub(crate) fn do_hatch(&self) -> Turtle {
        let response = self.do_command(Command::Hatch);
        if let Response::Turtle(t) = response {
            t
        } else {
            panic!("no turtle");
        }
    }

    fn req(&self, cmd: Command) -> Request {
        Request {
            turtle: self.turtle,
            thread: self.thread,
            cmd,
        }
    }

    fn do_command(&self, cmd: Command) -> Response {
        let is_data_cmd = matches!(cmd, Command::Data(_));
        let tracer_was_off = !*self.tracer.borrow();
        if let Command::Draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t))) = &cmd
        {
            *self.tracer.borrow_mut() = *t;
        }

        let cmd_string = format!("{cmd:?}");
        if self.issue_command.send(self.req(cmd)).is_ok() {
            if *self.tracer.borrow() {
                if tracer_was_off {
                    // need to consume all but the last response
                    let saved_response = self.command_complete.recv();
                    if let Ok(response) = saved_response {
                        let mut return_value = response;
                        loop {
                            match self.command_complete.try_recv() {
                                Ok(response) => return_value = response,
                                Err(TryRecvError::Empty) => return return_value,
                                Err(TryRecvError::Disconnected) => panic!("lost main thread"),
                            }
                        }
                    }
                } else if let Ok(result) = self.command_complete.recv() {
                    return result;
                }
            } else if is_data_cmd {
                loop {
                    if let Ok(result) = self.command_complete.recv() {
                        if !matches!(result, Response::Done) {
                            return result;
                        }
                    }
                }
            } else {
                loop {
                    match self.command_complete.try_recv() {
                        Ok(response) => {
                            assert!(
                                matches!(response, Response::Done),
                                "Received data response: {response:?} to command {cmd_string}"
                            );
                        }
                        Err(TryRecvError::Empty) => return Response::Done,
                        Err(TryRecvError::Disconnected) => panic!("lost main thread"),
                    }
                }
            }
        }

        /* main thread has gone away; wait here to meet our doom */
        loop {
            std::thread::park();
        }
    }
}

#[derive(Default, Debug)]
struct PolygonBuilder {
    last_point: Option<ScreenPosition<i32>>,
    verticies: Vec<[f32; 2]>,
}

impl PolygonBuilder {
    fn start(&mut self, pos: ScreenPosition<i32>) {
        self.last_point = Some(pos);
        self.verticies = vec![[to_f32(pos.x as isize), to_f32(pos.y as isize)]];
    }

    fn update(&mut self, pos: ScreenPosition<i32>) {
        if let Some(p) = self.last_point {
            if p != pos {
                let new_point = [to_f32(pos.x as isize), to_f32(pos.y as isize)];
                self.verticies.push(new_point);
                self.last_point = Some(pos);
            }
        }
    }

    fn close(&mut self) {
        if self.last_point.take().is_some() {
            self.verticies.push(self.verticies[0]);
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct DrawState {
    percent: f32,
    progression: Progression,
    insert_fill: Option<usize>,
    drawing_done: bool,
    tracer: bool,
    respond_immediately: bool,
    speed: Speed,
    current_stamp: usize,
    fill_poly: PolygonBuilder,
    shape_poly: PolygonBuilder,
    turtle: CurrentTurtleState,
}

impl DrawState {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug)]
struct TurtleTimer {
    time: Duration,
    prev: Instant,
    func: fn(&mut Turtle, Duration),
}

impl TurtleTimer {
    fn new(func: fn(&mut Turtle, Duration), time: Duration) -> Self {
        Self {
            time,
            prev: Instant::now(),
            func,
        }
    }
}

#[derive(Default, Debug)]
struct EventHandlers {
    onkeypress: HashMap<char, fn(&mut Turtle, char)>,
    onkeyrelease: HashMap<char, fn(&mut Turtle, char)>,
    onmousepress: Option<fn(&mut Turtle, x: f32, y: f32)>,
    onmouserelease: Option<fn(&mut Turtle, x: f32, y: f32)>,
    onmousedrag: Option<fn(&mut Turtle, x: f32, y: f32)>,
    ontimer: Option<TurtleTimer>,
    pending_keys: bool,
    requesting_thread: TurtleThread, // The thread that made the last drawing request
}

#[derive(Debug)]
struct TurtleData {
    queue: VecDeque<TurtleCommand>,       // new commands to draw
    current_command: Option<DrawRequest>, // what we're drawing now
    turtle_id: TurtleID,
    state: DrawState,
    event: EventHandlers,
    responder: HashMap<TurtleThread, Sender<Response>>,
    next_thread: TurtleThread,
}

impl TurtleData {
    fn new() -> Self {
        Self {
            state: DrawState {
                percent: 2.,
                tracer: true,
                ..DrawState::default()
            },
            queue: VecDeque::new(),
            current_command: None,
            turtle_id: TurtleID::default(),
            event: EventHandlers::default(),
            responder: HashMap::new(),
            next_thread: TurtleThread::default(),
        }
    }

    fn pending_key_event(&mut self) -> bool {
        if self.event.pending_keys {
            true
        } else {
            self.event.pending_keys = true;
            false
        }
    }

    fn spawn(&mut self, thread: TurtleThread, issue_command: Sender<Request>) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.responder.insert(thread, finished);

        Turtle::init(issue_command, command_complete, self.turtle_id, thread)
    }

    fn convert_command<G: TurtleGui>(&mut self, cmd: &DrawRequest, gui: &mut G) {
        if let Some(command) = self.state.turtle.apply(cmd) {
            let tid = self.turtle_id;

            if matches!(command, DrawCommand::Filler) {
                self.state.insert_fill = Some(gui.get_position(tid));
            }

            match &command {
                DrawCommand::Line(lineinfo) => {
                    // TODO: when "teleporting" instead of goto/setpos, we're only supposed
                    // to continue the current polygon if fill_gap=True (see python docs)
                    self.state.fill_poly.update(lineinfo.end);
                    self.state.shape_poly.update(lineinfo.end);
                    gui.append_command(tid, command);
                }
                DrawCommand::Circle(circle) => {
                    for c in circle {
                        self.state.fill_poly.update([c.x, c.y].into());
                        self.state.shape_poly.update([c.x, c.y].into());
                    }
                    gui.append_command(tid, command);
                }
                DrawCommand::DrawPolygon(_) => {
                    panic!("oops");
                }
                DrawCommand::StampTurtle => {
                    self.state.current_stamp =
                        gui.stamp(tid, self.state.turtle.pos(), self.state.turtle.angle);
                }
                DrawCommand::BeginPoly => {
                    let pos_copy = self.state.turtle.pos();
                    self.state.shape_poly.start(pos_copy);
                }
                DrawCommand::EndPoly => {
                    self.state.shape_poly.close();
                }
                DrawCommand::BeginFill => {
                    let pos_copy = self.state.turtle.pos();
                    self.state.fill_poly.start(pos_copy);
                    self.state.insert_fill = Some(gui.get_position(tid));
                    gui.append_command(tid, DrawCommand::Filler);
                }
                DrawCommand::EndFill => {
                    if !self.state.fill_poly.verticies.is_empty() {
                        let polygon = PolygonPath::new(&self.state.fill_poly.verticies);
                        self.state.fill_poly.last_point = None;
                        if let Some(index) = self.state.insert_fill.take() {
                            gui.fill_polygon(tid, DrawCommand::DrawPolygon(polygon), index);
                        }
                    }
                }
                DrawCommand::Clear => {
                    gui.clear_turtle(tid);
                    for cmd in self.state.turtle.get_state() {
                        gui.append_command(tid, cmd);
                    }
                }

                DrawCommand::Reset => {
                    gui.clear_turtle(tid);
                    self.state.turtle.reset();
                }

                DrawCommand::Filler
                | DrawCommand::Text(..)
                | DrawCommand::Filled(_)
                | DrawCommand::SetPenWidth(_)
                | DrawCommand::SetFillColor(_)
                | DrawCommand::SetPosition(_)
                | DrawCommand::SetHeading(..)
                | DrawCommand::Dot(..)
                | DrawCommand::DrawPolyAt(..)
                | DrawCommand::SetPenColor(_) => {
                    gui.append_command(tid, command);
                }
            }
        }
    }

    fn is_instantaneous(&self) -> bool {
        if let Some(cmd) = self.current_command.as_ref() {
            matches!(cmd, DrawRequest::InstantaneousDraw(_))
        } else {
            false
        }
    }

    fn time_passes<G: TurtleGui>(&mut self, gui: &mut G, delta_t: f32) {
        let s = self.state.speed.get();

        self.state.drawing_done = s == 0
            || match self.state.progression {
                Progression::Forward => self.state.percent >= 1.,
                Progression::Reverse => self.state.percent <= 0.,
            }
            || self.is_instantaneous();

        if self.state.drawing_done {
            self.state.percent = 1.;
        } else {
            let multiplier = f32::from(s);

            match self.state.progression {
                Progression::Forward => self.state.percent += delta_t * multiplier,
                Progression::Reverse => self.state.percent -= delta_t * multiplier,
            }
        }

        if !self.state.tracer && !self.queue.is_empty() {
            while !self.state.tracer && !self.queue.is_empty() {
                self.state.drawing_done = true;
                self.do_next_command(gui);
            }
        }
        self.do_next_command(gui);
    }

    fn do_next_command<G: TurtleGui>(&mut self, gui: &mut G) {
        if self.state.drawing_done && self.current_command.is_some() {
            self.state.drawing_done = false;

            if matches!(self.state.progression, Progression::Reverse) {
                if let Some(cmd) = gui.pop(self.turtle_id) {
                    self.state.turtle.undo(&cmd);
                }
            }

            let cmd = self.current_command.take().unwrap();

            if cmd.tracer_true() {
                self.state.respond_immediately = false;
            }

            if cmd.tracer_false() {
                self.state.respond_immediately = true;
            }

            if !self.state.respond_immediately {
                self.send_response(self.event.requesting_thread, cmd.is_stamp());
            }
        }

        if self.current_command.is_none() && !self.queue.is_empty() {
            let TurtleCommand {
                cmd,
                turtle,
                thread,
            } = self.queue.pop_front().unwrap();
            self.event.requesting_thread = thread;

            self.convert_command(&cmd, gui);

            if let DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t)) = &cmd {
                self.state.tracer = *t;
            }

            if matches!(cmd, DrawRequest::TimedDraw(TimedDrawCmd::Undo)) {
                self.state.progression = Progression::Reverse;
                self.state.percent = 1.;
                gui.undo(turtle);
            } else {
                self.state.progression = Progression::Forward;
                self.state.percent = 0.;
            }

            self.current_command = Some(cmd);
        }
    }

    fn send_response(&mut self, thread: TurtleThread, is_stamp: bool) {
        let _ = self.responder[&thread].send(if is_stamp {
            Response::StampID(self.state.current_stamp)
        } else {
            Response::Done
        });
    }

    fn reset(&mut self) {
        self.state.reset();
    }
}

type TurtleStartFunc = dyn FnOnce(&mut Turtle) + Send + 'static;

#[derive(Default)]
pub struct TurtleFlags {
    pub start_func: Option<Box<TurtleStartFunc>>,
    pub issue_command: Option<Sender<Request>>,
    pub receive_command: Option<Receiver<Request>>,
    pub title: String,
    pub size: [f32; 2],
}

#[allow(clippy::cast_precision_loss)]
fn to_f32<I: Into<isize>>(val: I) -> f32 {
    val.into() as f32
}
