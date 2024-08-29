pub(crate) mod types;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
};

use crate::{
    gui::{events::TurtleEvent, iced_gui::IcedGuiFramework, Progression, StampCount},
    turtle::types::TurtleID,
};

use either::Either;
use lyon_tessellation::geom::euclid::default::Transform2D;
use types::TurtleThread;

use crate::{
    color_names::TurtleColor,
    command::{
        Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd, TimedDrawCmd,
    },
    comms::{Request, Response},
    generate::{CurrentTurtleState, DrawCommand, TurtlePosition},
    polygon::{generate_default_shapes, TurtlePolygon, TurtleShape},
    speed::TurtleSpeed,
    ScreenPosition, TurtleShapeName,
};

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
pub(crate) struct TurtleInternalData {
    queue: VecDeque<TurtleCommand>, // new commands to draw

    current_command: Option<DrawRequest>, // what we're drawing now
    current_shape: CurrentTurtleState,
    current_stamp: usize,

    current_turtle: TurtleID,
    current_thread: TurtleThread,

    percent: f32,
    progression: Progression,
    insert_fill: Option<usize>,
    responder: HashMap<TurtleThread, Sender<Response>>,
    onkeypress: HashMap<char, fn(&mut Turtle, char)>,
    onkeyrelease: HashMap<char, fn(&mut Turtle, char)>,
    onmousepress: Option<fn(&mut Turtle, x: f32, y: f32)>,
    onmouserelease: Option<fn(&mut Turtle, x: f32, y: f32)>,
    onmousedrag: Option<fn(&mut Turtle, x: f32, y: f32)>,
    drawing_done: bool,
    tracer: bool,
    respond_immediately: bool,
    speed: TurtleSpeed,
    next_thread: TurtleThread,

    fill_poly: PolygonBuilder,
    shape_poly: PolygonBuilder,

    pending_keys: bool,
}

impl TurtleInternalData {
    fn pending_key_event(&mut self) -> bool {
        if self.pending_keys {
            true
        } else {
            self.pending_keys = true;
            false
        }
    }
}

use crate::gui::TurtleGui;
#[derive(Default)]
pub(crate) struct TurtleData {
    data: TurtleInternalData,
}

impl TurtleData {
    fn new() -> Self {
        Self {
            data: TurtleInternalData {
                percent: 2.,
                tracer: true,
                ..TurtleInternalData::default()
            },
        }
    }

    fn convert_command<G: TurtleGui>(&mut self, cmd: &DrawRequest, gui: &mut G) {
        if let Some(command) = self.data.current_shape.apply(cmd) {
            let tid = self.data.current_turtle;

            if matches!(command, DrawCommand::Filler) {
                self.data.insert_fill = Some(gui.get_position(tid))
            }

            match &command {
                DrawCommand::Line(lineinfo) => {
                    // TODO: when "teleporting" instead of goto/setpos, we're only supposed
                    // to continue the current polygon if fill_gap=True (see python docs)
                    self.data.fill_poly.update(lineinfo.end);
                    self.data.shape_poly.update(lineinfo.end);
                    gui.append_command(tid, command);
                }
                DrawCommand::Circle(circle) => {
                    for c in circle {
                        self.data.fill_poly.update([c.x, c.y].into());
                        self.data.shape_poly.update([c.x, c.y].into());
                    }
                    gui.append_command(tid, command);
                }
                DrawCommand::DrawPolygon(_) => {
                    panic!("oops");
                }
                DrawCommand::StampTurtle => {
                    self.data.current_stamp = gui.stamp(
                        tid,
                        self.data.current_shape.pos(),
                        self.data.current_shape.angle,
                    );
                }
                DrawCommand::BeginPoly => {
                    let pos_copy = self.data.current_shape.pos();
                    self.data.shape_poly.start(pos_copy);
                }
                DrawCommand::EndPoly => {
                    self.data.shape_poly.close();
                }
                DrawCommand::BeginFill => {
                    let pos_copy = self.data.current_shape.pos();
                    self.data.fill_poly.start(pos_copy);
                    self.data.insert_fill = Some(gui.get_position(tid));
                    gui.append_command(tid, DrawCommand::Filler);
                }
                DrawCommand::EndFill => {
                    if !self.data.fill_poly.verticies.is_empty() {
                        let polygon = TurtlePolygon::new(&self.data.fill_poly.verticies);
                        self.data.fill_poly.last_point = None;
                        if let Some(index) = self.data.insert_fill.take() {
                            gui.fill_polygon(tid, DrawCommand::DrawPolygon(polygon), index);
                        }
                    }
                }
                _ => {
                    gui.append_command(tid, command);
                }
            }
        }
    }

    fn is_instantaneous(&self) -> bool {
        if let Some(cmd) = self.data.current_command.as_ref() {
            matches!(cmd, DrawRequest::InstantaneousDraw(_))
        } else {
            false
        }
    }

    fn time_passes<G: TurtleGui>(&mut self, gui: &mut G, delta_t: f32) {
        let s = self.data.speed.get();

        self.data.drawing_done = s == 0
            || match self.data.progression {
                Progression::Forward => self.data.percent >= 1.,
                Progression::Reverse => self.data.percent <= 0.,
            }
            || self.is_instantaneous();

        if self.data.drawing_done {
            self.data.percent = 1.;
        } else {
            let multiplier = s as f32;

            match self.data.progression {
                Progression::Forward => self.data.percent += delta_t * multiplier,
                Progression::Reverse => self.data.percent -= delta_t * multiplier,
            }
        }

        if !self.data.tracer && !self.data.queue.is_empty() {
            while !self.data.tracer && !self.data.queue.is_empty() {
                self.data.drawing_done = true;
                self.do_next_command(gui);
            }
        }
        self.do_next_command(gui);
    }

    fn do_next_command<G: TurtleGui>(&mut self, gui: &mut G) {
        if self.data.drawing_done && self.data.current_command.is_some() {
            self.data.drawing_done = false;

            if matches!(self.data.progression, Progression::Reverse) {
                if let Some(cmd) = gui.pop(self.data.current_turtle) {
                    match cmd {
                        DrawCommand::Line(line) => {
                            let x = line.begin.x as f32;
                            let y = line.begin.y as f32;
                            self.data.current_shape.transform = Transform2D::translation(x, y);
                        }
                        DrawCommand::SetHeading(start, _) => {
                            self.data.current_shape.angle = start;
                        }
                        _ => {}
                    }
                }
            }

            let cmd = self.data.current_command.take().unwrap();

            if cmd.tracer_true() {
                self.data.respond_immediately = false;
            }

            if !self.data.respond_immediately {
                self.send_response(self.data.current_thread, cmd.is_stamp());
            }

            if cmd.tracer_false() {
                self.data.respond_immediately = true;
            }
        }

        if self.data.current_command.is_none() && !self.data.queue.is_empty() {
            let TurtleCommand {
                cmd,
                turtle,
                thread,
            } = self.data.queue.pop_front().unwrap();
            self.data.current_turtle = turtle;
            self.data.current_thread = thread;

            self.convert_command(&cmd, gui);

            if let DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t)) = &cmd {
                self.data.tracer = *t;
            }

            if matches!(cmd, DrawRequest::TimedDraw(TimedDrawCmd::Undo)) {
                self.data.progression = Progression::Reverse;
                self.data.percent = 1.;
                gui.undo(turtle);
            } else {
                self.data.progression = Progression::Forward;
                self.data.percent = 0.;
            }

            self.data.current_command = Some(cmd);
        }
    }

    fn send_response(&mut self, thread: TurtleThread, is_stamp: bool) {
        let _ = self.data.responder[&thread].send(if is_stamp {
            Response::StampID(self.data.current_stamp)
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
    data: Vec<TurtleData>,
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
            data: vec![TurtleData::new()],
            shapes: generate_default_shapes(),
            ..Self::default()
        }
    }

    pub(crate) fn progress(&self, tid: TurtleID) -> (f32, Progression) {
        (self.data[tid].data.percent, self.data[tid].data.progression)
    }

    pub(crate) fn popup_result(
        &mut self,
        turtle: TurtleID,
        thread: TurtleThread,
        response: Response,
    ) {
        let _ = self.data[turtle].data.responder[&thread].send(response);
    }

    pub(crate) fn popup_cancelled(&mut self, turtle: TurtleID, thread: TurtleThread) {
        let _ = self.data[turtle].data.responder[&thread].send(Response::Cancel);
    }

    pub(crate) fn handle_event(
        &mut self,
        turtle: Option<TurtleID>,
        thread: Option<TurtleThread>,
        event: TurtleEvent,
    ) {
        use TurtleEvent::*;

        let mut work = Vec::new();

        match event {
            WindowResize(width, height) => {
                self.winsize = [width as isize, height as isize];
                if turtle.is_none() {
                    assert!(thread.is_none());
                } else {
                    let turtle = turtle.expect("missing turtle from window resize");
                    let thread = thread.expect("missing thread from window resize");
                    let _ = self.data[turtle].data.responder[&thread].send(Response::Done);
                }
            }
            KeyPress(ch) => {
                for (idx, turtle) in self.data.iter_mut().enumerate() {
                    if let Some(func) = turtle.data.onkeypress.get(&ch).copied() {
                        if !turtle.data.pending_key_event() {
                            work.push((TurtleID::new(idx), Either::Right((func, ch))));
                        }
                    }
                }
            }
            KeyRelease(ch) => {
                for (idx, turtle) in self.data.iter_mut().enumerate() {
                    if let Some(func) = turtle.data.onkeyrelease.get(&ch).copied() {
                        if !turtle.data.pending_key_event() {
                            work.push((TurtleID::new(idx), Either::Right((func, ch))));
                        }
                    }
                }
            }
            MousePress(x, y) => {
                for (idx, turtle) in self.data.iter().enumerate() {
                    if let Some(func) = turtle.data.onmousepress {
                        work.push((TurtleID::new(idx), Either::Left((func, x, y))));
                    }
                }
            }
            MouseRelease(x, y) => {
                for (idx, turtle) in self.data.iter().enumerate() {
                    if let Some(func) = turtle.data.onmouserelease {
                        work.push((TurtleID::new(idx), Either::Left((func, x, y))));
                    }
                }
            }
            MousePosition(_, _) => todo!(),
            MouseDrag(x, y) => {
                for (idx, turtle) in self.data.iter().enumerate() {
                    if let Some(func) = turtle.data.onmousedrag {
                        work.push((TurtleID::new(idx), Either::Left((func, x, y))));
                    }
                }
            }
            _Timer => todo!(),
            Unhandled => {}
        }

        for (idx, job) in work {
            let tid = self.data[idx].data.next_thread.get();
            let mut new_turtle = self.spawn_turtle(idx, tid);

            let _ = std::thread::spawn(move || {
                match job {
                    Either::Left((func, x, y)) => func(&mut new_turtle, x, y),
                    Either::Right((func, key)) => func(&mut new_turtle, key),
                }
                let _ = new_turtle.issue_command.send(Request::shut_down(idx, tid));
            });
        }
    }

    pub(crate) fn run_turtle<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let mut turtle = self.spawn_turtle(TurtleID::new(0), TurtleThread::new(0));
        let _ = std::thread::spawn(move || func(&mut turtle));
    }

    pub(crate) fn tick<G: TurtleGui>(&mut self, gui: &mut G) {
        while let Ok(req) = self.receive_command.as_ref().unwrap().try_recv() {
            self.handle_command(req, gui);
        }

        for turtle in self.data.iter_mut() {
            turtle.time_passes(gui, 0.01); // TODO: use actual time delta
        }
    }

    pub(crate) fn hatch_turtle<G: TurtleGui>(&mut self, gui: &mut G) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        let turtle = gui.new_turtle();
        let thread = TurtleThread::new(0);

        let mut td = TurtleData::new();
        td.data.responder.insert(thread, finished);
        self.data.push(td);

        Turtle::init(
            self.issue_command.as_ref().unwrap().clone(),
            command_complete,
            turtle,
            thread,
        )
    }

    fn spawn_turtle(&mut self, turtle: TurtleID, thread: TurtleThread) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.data[turtle].data.responder.insert(thread, finished);

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
        let resp = self.data[turtle]
            .data
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
                self.data[turtle].data.speed = s;
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
        let resp = self.data[turtle]
            .data
            .responder
            .get(&thread)
            .unwrap()
            .clone();
        match cmd {
            InputCmd::KeyRelease(f, k) => {
                self.data[turtle].data.onkeyrelease.insert(k, f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::KeyPress(f, k) => {
                self.data[turtle].data.onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MouseDrag(f) => {
                self.data[turtle].data.onmousedrag = Some(f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MousePress(f) => {
                self.data[turtle].data.onmousepress = Some(f);
                let _ = resp.send(Response::Done);
            }
            InputCmd::MouseRelease(f) => {
                self.data[turtle].data.onmouserelease = Some(f);
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
        let resp = self.data[turtle]
            .data
            .responder
            .get(&thread)
            .unwrap()
            .clone();

        let _ = match &cmd {
            DataCmd::GetScreenSize => resp.send(Response::ScreenSize(self.winsize)),
            DataCmd::Visibility => resp.send(Response::Visibility(gui.is_visible(turtle))),
            DataCmd::GetPoly => resp.send(Response::Polygon(
                self.data[turtle].data.shape_poly.verticies.clone(),
            )),
            DataCmd::TurtleShape(shape) => {
                if let TurtleShapeName::Shape(name) = shape {
                    gui.set_shape(turtle, self.shapes[name].clone());
                }
                resp.send(Response::Name(gui.get_turtle_shape_name(turtle)))
            }
            DataCmd::UndoBufferEntries => resp.send(Response::Count(gui.undo_count(turtle))),
            DataCmd::Towards(xpos, ypos) => {
                let curpos: ScreenPosition<f32> = self.data[turtle].data.current_shape.pos();
                let x = xpos - curpos.x;
                let y = ypos + curpos.y;
                let heading = y.atan2(x) * 360. / (2.0 * PI);

                resp.send(Response::Heading(heading))
            }
            DataCmd::Position => resp.send(Response::Position(
                self.data[turtle].data.current_shape.pos(),
            )),
            DataCmd::Heading => resp.send(Response::Heading(
                self.data[turtle].data.current_shape.angle(),
            )),
            DataCmd::Stamp => {
                self.data[turtle].data.queue.push_back(TurtleCommand {
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
        self.data[turtle].data.queue.push_back(TurtleCommand {
            cmd,
            turtle,
            thread,
        });

        // FIXME: data commands (Command::Data(_)) require all queued entries to be
        // processed before sending a response, even if `respond_immediately` is set
        if self.data[turtle].data.respond_immediately {
            self.data[turtle].send_response(thread, is_stamp);
        }
    }

    fn handle_command<G: TurtleGui>(&mut self, req: Request, gui: &mut G) {
        let turtle = req.turtle;
        let thread = req.thread;

        match req.cmd {
            Command::ShutDown => {
                let tid = self.data[turtle].data.responder.remove(&thread);
                self.data[turtle].data.pending_keys = false;
                assert!(tid.is_some());
            }
            Command::Screen(cmd) => self.screen_cmd(turtle, cmd, thread, gui),
            Command::Draw(cmd) => self.draw_cmd(turtle, cmd, thread),
            Command::Input(cmd) => self.input_cmd(turtle, cmd, thread),
            Command::Data(cmd) => self.data_cmd(turtle, cmd, thread, gui),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle(gui);
                let resp = &self.data[turtle].data.responder[&thread];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }
}
