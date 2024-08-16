use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
};

use crate::gui::{iced_gui::IcedGui, Progression};

use iced::keyboard::{Event::KeyPressed, Key};
use iced::window::Event::Resized;
use iced::{window, Event};
use lyon_tessellation::geom::euclid::default::Transform2D;

use either::Either;

use crate::{
    color_names::TurtleColor,
    command::{
        Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd, TimedDrawCmd,
    },
    generate::{CurrentTurtleState, DrawCommand, TurtlePosition},
    polygon::{generate_default_shapes, TurtlePolygon, TurtleShape},
    speed::TurtleSpeed,
    Request, Response, ScreenPosition, TurtleShapeName,
};

//
// whenver we get a new command from the turtle, we're going to add to
// the Turtle::iced_draw_commands vec. We need to convert the command into
// drawing coordinates first (see the CurrentTurtleState::apply function),
// and then from those coordinates into Iced Stroke/Fill for paths.
//
// Once the conversion is done, we should be able just draw in a fast loop.
//
// Complications:
// 1/ When we draw lines, or move the turtle, we want to see it happen via
//    animation. This means that the most recent command happens over a period
//    of time.
//
// 2/ When we get a new command, we want to see if we can "attach" it to a
//    previous command. For example, two lines in a row, separated by a
//    rotation, should be able to be a single path with a single stroke.
//
// 3/ If the user modifies the pencolor/penwidth values, then we need to start
//    a new path/stroke.
//
// Maintain a list of "completed" drawing commands
// Keep one "current" drawing command
//
// Sequence of events:
//  1/ Turtle sends a DrawRequest to the main thread
//  2/ Main thread puts it on a queue
//  3/ When the command is popped from the queue, call apply(DrawRequest).
//     This returns a DrawCommand, which is put into the 'elements' vec
//  4/ convert_to_iced() loops over the entire elements array and generates
//     the IcedDrawCmd objects.
//  5/ iced->draw will loop over the IcedDrawCmd objects and draw them in order
//     on the display
//
//  1..3 -> Same
//  4 -> need a "currently in progress" IcedDrawCmd, plus all the previously
//       converted DrawCommand objects in an array
//       - iced->draw will draw all the converted ones, and then draw the
//         the final "in progress" one

pub(crate) type TurtleID = usize;

#[derive(Debug)]
struct TurtleCommand {
    cmd: DrawRequest,
    turtle_id: TurtleID,
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
    turtle_id: TurtleID,
    tracer: RefCell<bool>,
}

enum ClearDirection {
    Forward,
    Reverse,
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
        IcedGui::start(flags);
    }

    pub(crate) fn init(
        issue_command: Sender<Request>,
        command_complete: Receiver<Response>,
        turtle_id: TurtleID,
    ) -> Self {
        Self {
            issue_command,
            command_complete,
            turtle_id,
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
            turtle_id: self.turtle_id,
            cmd,
        }
    }

    fn do_command(&self, cmd: Command) -> Response {
        let is_data_cmd = matches!(cmd, Command::Data(_));
        let tracer_was_off = *self.tracer.borrow();
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
                            println!("Got response: {result:?}");
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
    queue: VecDeque<TurtleCommand>,       // new elements to draw
    current_command: Option<DrawRequest>, // what we're drawing now
    elements: Vec<DrawCommand>,
    current_shape: CurrentTurtleState,

    current_turtle_id: TurtleID, // which thread to notify on completion
    percent: f32,
    progression: Progression,
    insert_fill: Option<usize>,
    responder: HashMap<TurtleID, Sender<Response>>,
    onkeypress: HashMap<char, fn(&mut Turtle, char)>,
    drawing_done: bool,
    turtle_invisible: bool,
    tracer: bool,
    respond_immediately: bool,
    speed: TurtleSpeed,
    turtle_shape: TurtleShape,
    fill_poly: PolygonBuilder,
    shape_poly: PolygonBuilder,
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
            let tid = self.data.current_turtle_id;

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
                    // self.data.elements.push(command);
                }
                DrawCommand::Circle(circle) => {
                    for c in circle {
                        self.data.fill_poly.update([c.x, c.y].into());
                        self.data.shape_poly.update([c.x, c.y].into());
                    }
                    gui.append_command(tid, command);
                }
                DrawCommand::DrawPolygon(_) => {
                    if let Some(index) = self.data.insert_fill.take() {
                        gui.fill_polygon(tid, command, index);
                    }
                }
                DrawCommand::StampTurtle => {
                    gui.append_command(
                        tid,
                        DrawCommand::DrawPolyAt(
                            self.data.turtle_shape.shape.clone(),
                            self.data.current_shape.pos(),
                            self.data.current_shape.angle,
                        ),
                    );
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
                gui.undo(self.data.current_turtle_id);
                if let Some(element) = self.data.elements.pop() {
                    match element {
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
                self.send_response(self.data.current_turtle_id, cmd.is_stamp());
            }

            if cmd.tracer_false() {
                self.data.respond_immediately = true;
            }
        }

        if self.data.current_command.is_none() && !self.data.queue.is_empty() {
            let TurtleCommand { cmd, turtle_id } = self.data.queue.pop_front().unwrap();
            self.data.current_turtle_id = turtle_id;

            self.convert_command(&cmd, gui);

            if let DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t)) = &cmd {
                self.data.tracer = *t;
            }

            if matches!(cmd, DrawRequest::TimedDraw(TimedDrawCmd::Undo)) {
                self.data.progression = Progression::Reverse;
                self.data.percent = 1.;
            } else {
                self.data.progression = Progression::Forward;
                self.data.percent = 0.;
            }

            self.data.current_command = Some(cmd);
        }
    }

    fn send_response(&mut self, turtle_id: TurtleID, is_stamp: bool) {
        let _ = self.data.responder[&turtle_id].send(if is_stamp {
            Response::StampID(self.data.elements.len() - 1)
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

/*
impl Application for TurtleTask {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = TurtleFlags;

    fn view(&self, win_id: WindowID) -> Element<Self::Message> {
    }

}
*/

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

    pub(crate) fn popup_result(&mut self, tid: usize, index: usize, response: Response) {
        let _ = self.data[index].data.responder[&tid].send(response);
    }

    pub(crate) fn popup_cancelled(&mut self, tid: usize, index: usize) {
        let _ = self.data[index].data.responder[&tid].send(Response::Cancel);
    }

    pub(crate) fn handle_event<G: TurtleGui>(&mut self, event: Event, gui: &mut G) {
        let mut work = Vec::new();

        match event {
            Event::Window(window::Id::MAIN, Resized { width, height }) => {
                self.winsize = [width as isize, height as isize];
            }
            Event::Keyboard(KeyPressed { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    for (idx, turtle) in self.data.iter().enumerate() {
                        if let Some(func) = turtle.data.onkeypress.get(&ch).copied() {
                            work.push((idx, func, ch));
                        }
                    }
                }
            }
            _ => {}
        }

        for (idx, func, key) in work {
            let mut new_turtle = self.spawn_turtle(idx, gui.new_turtle());
            let _ = std::thread::spawn(move || func(&mut new_turtle, key));
        }
    }

    pub(crate) fn run_turtle<F: FnOnce(&mut Turtle) + Send + 'static>(
        &mut self,
        func: F,
        newid: TurtleID,
    ) {
        let mut turtle = self.spawn_turtle(0, newid);
        let _ = std::thread::spawn(move || func(&mut turtle));
    }

    pub(crate) fn tick<G: TurtleGui>(&mut self, gui: &mut G) {
        while let Ok(req) = self.receive_command.as_ref().unwrap().try_recv() {
            let tid = req.turtle_id;
            let mut found = None;
            for (index, tdata) in self.data.iter().enumerate() {
                if tdata.data.responder.contains_key(&tid) {
                    found = Some(index);
                    break;
                }
            }
            if let Some(index) = found {
                self.handle_command(index, req, gui);
            }
        }

        for turtle in self.data.iter_mut() {
            turtle.time_passes(gui, 0.01); // TODO: use actual time delta
        }
    }

    pub(crate) fn hatch_turtle<G: TurtleGui>(&mut self, gui: &mut G) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        let newid = gui.new_turtle();

        let mut td = TurtleData::new();
        td.data.responder.insert(newid, finished);
        self.data.push(td);

        Turtle::init(
            self.issue_command.as_ref().unwrap().clone(),
            command_complete,
            newid,
        )
    }

    fn spawn_turtle(&mut self, which: usize, newid: TurtleID) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.data[which].data.responder.insert(newid, finished);

        Turtle::init(
            self.issue_command.as_ref().unwrap().clone(),
            command_complete,
            newid,
        )
    }

    fn screen_cmd(&mut self, which: usize, cmd: ScreenCmd, turtle_id: TurtleID) {
        let resp = self.data[which]
            .data
            .responder
            .get(&turtle_id)
            .unwrap()
            .clone();
        match cmd {
            ScreenCmd::SetSize(s) => {
                /* TODO: move to gui
                self.wcmds
                    .push(window::resize::<Message>(window::Id::MAIN, s));
                */
                self.winsize = s; // TODO: wait until resize is complete before saving
                let _ = resp.send(Response::Done); // TODO: don't respond until resize event
            }
            ScreenCmd::ShowTurtle(t) => {
                self.data[which].data.turtle_invisible = !t;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Speed(s) => {
                self.data[which].data.speed = s;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginPoly => {
                let pos_copy = self.data[which].data.current_shape.pos();
                self.data[which].data.shape_poly.start(pos_copy);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::EndPoly => {
                self.data[which].data.shape_poly.close();
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginFill => {
                let pos_copy = self.data[which].data.current_shape.pos();
                self.data[which].data.fill_poly.start(pos_copy);
                self.data[which].data.queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon),
                    turtle_id,
                });
            }
            ScreenCmd::EndFill => {
                if !self.data[which].data.fill_poly.verticies.is_empty() {
                    let polygon = TurtlePolygon::new(&self.data[which].data.fill_poly.verticies);
                    self.data[which].data.fill_poly.last_point = None;
                    self.data[which].data.queue.push_back(TurtleCommand {
                        cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Fill(polygon)),
                        turtle_id,
                    })
                }
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.bgcolor = [r, g, b, 1.].into();
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data[which].data.elements.clear();
                self.bgcolor = TurtleColor::from("black");
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data[which].data.elements.len()
                    && matches!(
                        self.data[which].data.elements[id],
                        DrawCommand::DrawPolyAt(..)
                    )
                {
                    self.data[which].data.elements[id] = DrawCommand::Filler;
                }
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamps(count) => {
                #[allow(clippy::comparison_chain)]
                if count < 0 {
                    self.clear_stamps(which, -count, ClearDirection::Reverse);
                } else if count == 0 {
                    self.clear_stamps(which, isize::MAX, ClearDirection::Forward);
                } else {
                    self.clear_stamps(which, count, ClearDirection::Forward);
                }
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn clear_stamps(&mut self, which: usize, mut count: isize, dir: ClearDirection) {
        let mut iter = match dir {
            ClearDirection::Forward => Either::Right(self.data[which].data.elements.iter_mut()),
            ClearDirection::Reverse => {
                Either::Left(self.data[which].data.elements.iter_mut().rev())
            }
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if cmd.is_stamp() {
                    count -= 1;
                    *cmd = DrawCommand::Filler
                }
            } else {
                break;
            }
        }
    }

    fn input_cmd(&mut self, which: usize, cmd: InputCmd, turtle_id: TurtleID) {
        let resp = self.data[which]
            .data
            .responder
            .get(&turtle_id)
            .unwrap()
            .clone();
        match cmd {
            InputCmd::OnKeyPress(f, k) => {
                self.data[which].data.onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn data_cmd<G: TurtleGui>(
        &mut self,
        which: usize,
        cmd: DataCmd,
        turtle_id: TurtleID,
        gui: &mut G,
    ) {
        let resp = self.data[which]
            .data
            .responder
            .get(&turtle_id)
            .unwrap()
            .clone();
        let _ = match &cmd {
            DataCmd::GetScreenSize => resp.send(Response::ScreenSize(self.winsize)),
            DataCmd::Visibility => resp.send(Response::Visibility(
                !self.data[which].data.turtle_invisible,
            )),
            DataCmd::GetPoly => resp.send(Response::Polygon(
                self.data[which].data.shape_poly.verticies.clone(),
            )),
            DataCmd::TurtleShape(shape) => {
                if let TurtleShapeName::Shape(name) = shape {
                    self.data[which].data.turtle_shape = self.shapes[name].clone();
                }
                resp.send(Response::Name(
                    self.data[which].data.turtle_shape.name.clone(),
                ))
            }
            DataCmd::UndoBufferEntries => resp.send(Response::Count(gui.undo_count(which))),
            DataCmd::Towards(xpos, ypos) => {
                let curpos: ScreenPosition<f32> = self.data[which].data.current_shape.pos();
                let x = xpos - curpos.x;
                let y = ypos + curpos.y;
                let heading = y.atan2(x) * 360. / (2.0 * PI);

                resp.send(Response::Heading(heading))
            }
            DataCmd::Position => resp.send(Response::Position(
                self.data[which].data.current_shape.pos(),
            )),
            DataCmd::Heading => resp.send(Response::Heading(
                self.data[which].data.current_shape.angle(),
            )),
            DataCmd::Stamp => {
                self.data[which].data.queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Stamp),
                    turtle_id,
                });
                Ok(())
            }
            DataCmd::NumInput(title, prompt) => {
                gui.numinput(turtle_id, which, title, prompt);
                Ok(())
            }
            DataCmd::TextInput(title, prompt) => {
                gui.textinput(turtle_id, which, title, prompt);
                Ok(())
            }
        };
    }

    fn draw_cmd(&mut self, which: usize, cmd: DrawRequest, turtle_id: TurtleID) {
        let is_stamp = cmd.is_stamp();
        self.data[which]
            .data
            .queue
            .push_back(TurtleCommand { cmd, turtle_id });

        // FIXME: data commands (Command::Data(_)) require all queued entries to be
        // processed before sending a response, even if `respond_immediately` is set
        if self.data[which].data.respond_immediately {
            self.data[which].send_response(turtle_id, is_stamp);
        }
    }

    fn handle_command<G: TurtleGui>(&mut self, which: usize, req: Request, gui: &mut G) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(which, cmd, req.turtle_id),
            Command::Draw(cmd) => self.draw_cmd(which, cmd, req.turtle_id),
            Command::Input(cmd) => self.input_cmd(which, cmd, req.turtle_id),
            Command::Data(cmd) => self.data_cmd(which, cmd, req.turtle_id, gui),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle(gui);
                let resp = &self.data[which].data.responder[&req.turtle_id];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }
}
