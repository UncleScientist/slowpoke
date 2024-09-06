pub(crate) mod types;
use types::TurtleThread;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    time::{Duration, Instant},
};

use crate::{
    color_names::TurtleColor,
    command::{
        Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd, TimedDrawCmd,
    },
    comms::{Request, Response},
    generate::{CurrentTurtleState, DrawCommand, TurtlePosition},
    gui::{events::TurtleEvent, iced_gui::IcedGuiFramework, Progression, StampCount, TurtleGui},
    polygon::{generate_default_shapes, TurtlePolygon, TurtleShape},
    speed::TurtleSpeed,
    turtle::types::TurtleID,
    ScreenPosition, TurtleShapeName,
};

macro_rules! spawn {
    ($task: expr, $td:expr, $idx:expr, $func:expr, $($args:tt)*) => {{
        let _turtle :TurtleID = $idx.into();
        let _thread = $td.next_thread.get();

        let mut _new_turtle = $td.spawn(_thread,
            $task.issue_command.as_ref().unwrap().clone());

        let _ = std::thread::spawn(move || {
            $func(&mut _new_turtle, $($args)*);
            let _ = _new_turtle.issue_command.send(Request::shut_down(_turtle, _thread));
        });
    }};
}

#[derive(Debug)]
struct TurtleCommand {
    cmd: DrawRequest,
    turtle: TurtleID,
    thread: TurtleThread,
}

pub struct TurtleArgs {
    pub(crate) size: [isize; 2],
    pub(crate) title: String,
}

impl Default for TurtleArgs {
    fn default() -> Self {
        Self {
            size: [800, 800],
            title: "Turtle".to_string(),
        }
    }
}

impl TurtleArgs {
    pub fn with_size(mut self, x: isize, y: isize) -> Self {
        self.size = [x, y];
        self
    }
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn run<F: FnOnce(&mut Turtle) + Send + 'static>(&self, func: F) {
        Turtle::run(self, func)
    }
}

#[derive(Debug)]
pub struct Turtle {
    issue_command: Sender<Request>,
    command_complete: Receiver<Response>,
    turtle: TurtleID,
    thread: TurtleThread,
    tracer: RefCell<bool>,
}

impl Turtle {
    #[allow(clippy::new_ret_no_self)] // TODO: fix this
    pub fn new() -> TurtleArgs {
        TurtleArgs::default()
    }

    pub fn run<F: FnOnce(&mut Turtle) + Send + 'static>(args: &TurtleArgs, func: F) {
        let xsize = args.size[0] as f32;
        let ysize = args.size[1] as f32;

        let (issue_command, receive_command) = mpsc::channel();

        let flags = TurtleFlags {
            start_func: Some(Box::new(func)),
            issue_command: Some(issue_command),
            receive_command: Some(receive_command),
            title: args.title.clone(),
            size: [xsize, ysize],
        };

        // #[cfg(an option to specify the "iced" crate for the gui)]
        IcedGuiFramework::start(flags);

        // EguiGui::start(flags);
        // SlintGui::start(flags);
        // OtherGui::start(flags);
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
                        if matches!(result, Response::Done) {
                            continue;
                        } else {
                            return result;
                        }
                    }
                }
            } else {
                loop {
                    match self.command_complete.try_recv() {
                        Ok(response) => {
                            if matches!(response, Response::Done) {
                                continue;
                            } else {
                                panic!("Received data response: {response:?}");
                            }
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

#[derive(Default)]
struct PolygonBuilder {
    last_point: Option<ScreenPosition<isize>>,
    verticies: Vec<[f32; 2]>,
}

impl PolygonBuilder {
    fn start(&mut self, pos: ScreenPosition<isize>) {
        self.last_point = Some(pos);
        self.verticies = vec![[pos.x as f32, pos.y as f32]];
    }

    fn update(&mut self, pos: ScreenPosition<isize>) {
        if let Some(p) = self.last_point {
            if p != pos {
                let new_point = [pos.x as f32, pos.y as f32];
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

#[derive(Default)]
struct DrawState {
    percent: f32,
    progression: Progression,
    insert_fill: Option<usize>,
    drawing_done: bool,
    tracer: bool,
    respond_immediately: bool,
    speed: TurtleSpeed,
    current_stamp: usize,
    fill_poly: PolygonBuilder,
    shape_poly: PolygonBuilder,
    turtle: CurrentTurtleState,
}

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

#[derive(Default)]
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

#[derive(Default)]
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
            ..Self::default()
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
                self.state.insert_fill = Some(gui.get_position(tid))
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
                        let polygon = TurtlePolygon::new(&self.state.fill_poly.verticies);
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
                _ => {
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
            let multiplier = s as f32;

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

            if !self.state.respond_immediately {
                self.send_response(self.event.requesting_thread, cmd.is_stamp());
            }

            if cmd.tracer_false() {
                self.state.respond_immediately = true;
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
}

#[derive(Default)]
pub(crate) struct TurtleTask {
    issue_command: Option<Sender<Request>>,
    receive_command: Option<Receiver<Request>>,
    bgcolor: TurtleColor,
    turtle_list: Vec<TurtleData>,
    shapes: HashMap<String, TurtleShape>,
    winsize: [isize; 2],
}

type TurtleStartFunc = dyn FnOnce(&mut Turtle) + Send + 'static;

#[derive(Default)]
pub(crate) struct TurtleFlags {
    pub(crate) start_func: Option<Box<TurtleStartFunc>>,
    pub(crate) issue_command: Option<Sender<Request>>,
    pub(crate) receive_command: Option<Receiver<Request>>,
    pub(crate) title: String,
    pub(crate) size: [f32; 2],
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
        event: TurtleEvent,
    ) {
        use TurtleEvent::*;

        match event {
            WindowResize(width, height) => {
                self.winsize = [width as isize, height as isize];
                if turtle.is_none() {
                    assert!(thread.is_none());
                } else {
                    let turtle = turtle.expect("missing turtle from window resize");
                    let thread = thread.expect("missing thread from window resize");
                    let _ = self.turtle_list[turtle].responder[&thread].send(Response::Done);
                }
            }
            KeyPress(ch) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onkeypress.get(&ch).copied() {
                        if !turtle.pending_key_event() {
                            spawn!(self, turtle, idx, func, ch);
                        }
                    }
                }
            }
            KeyRelease(ch) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onkeyrelease.get(&ch).copied() {
                        if !turtle.pending_key_event() {
                            spawn!(self, turtle, idx, func, ch);
                        }
                    }
                }
            }
            MousePress(x, y) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmousepress {
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            MouseRelease(x, y) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmouserelease {
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            MousePosition(_, _) => todo!(),
            MouseDrag(x, y) => {
                for (idx, turtle) in self.turtle_list.iter_mut().enumerate() {
                    if let Some(func) = turtle.event.onmousedrag {
                        spawn!(self, turtle, idx, func, x, y);
                    }
                }
            }
            _Timer => todo!(),
            Unhandled => {}
        }
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

        for turtle in self.turtle_list.iter_mut() {
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
            ScreenCmd::SetSize(s) => {
                gui.resize(turtle, thread, s[0], s[1]);
                // Note: don't send "done" here -- wait for the resize event from the GUI
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
                self.bgcolor = TurtleColor::from("black");
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                gui.clear_stamp(turtle, id);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamps(count) => {
                #[allow(clippy::comparison_chain)]
                if count < 0 {
                    gui.clear_stamps(turtle, StampCount::Reverse((-count) as usize));
                } else if count == 0 {
                    gui.clear_stamps(turtle, StampCount::All);
                } else {
                    gui.clear_stamps(turtle, StampCount::Forward(count as usize));
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
        cmd: DataCmd,
        thread: TurtleThread,
        gui: &mut G,
    ) {
        let resp = self.turtle_list[turtle]
            .responder
            .get(&thread)
            .unwrap()
            .clone();

        let _ = match &cmd {
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
                    gui.set_shape(turtle, self.shapes[name].clone());
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
            Command::Data(cmd) => self.data_cmd(turtle, cmd, thread, gui),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle(gui);
                let resp = &self.turtle_list[turtle].responder[&thread];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }
}
