use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender},
};

use either::Either;
use glutin_window::GlutinWindow;
use graphics::types::{self, Vec2d};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, ButtonArgs, ButtonEvent, ButtonState, EventSettings, Events, Key, RenderArgs,
    RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};

use crate::{
    color_names::TurtleColor,
    command::{
        Command, DataCmd, DrawCmd, InputCmd, InstantaneousDrawCmd, ScreenCmd, TurtleDrawState,
    },
    polygon::TurtlePolygon,
    speed::TurtleSpeed,
    Request, Response,
};

#[derive(Debug)]
struct DrawRequest {
    cmd: DrawCmd,
    turtle_id: u64,
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
    turtle_id: u64,
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
        let xsize: f64 = args.size[0] as f64;
        let ysize: f64 = args.size[1] as f64;

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create a Glutin window.
        let window: GlutinWindow = WindowSettings::new(&args.title, [xsize, ysize])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .samples(8)
            .build()
            .unwrap();

        let (issue_command, receive_command) = mpsc::channel();
        let mut tt = TurtleTask {
            gl: GlGraphics::new(opengl),
            window,
            issue_command,
            receive_command,
            turtle_num: 0,
            bgcolor: crate::WHITE,
            data: vec![TurtleData {
                percent: 2.,
                ..TurtleData::default()
            }],
        };

        tt.run(func);
    }

    pub(crate) fn init(
        issue_command: Sender<Request>,
        command_complete: Receiver<Response>,
        turtle_id: u64,
    ) -> Self {
        Self {
            issue_command,
            command_complete,
            turtle_id,
        }
    }

    pub(crate) fn do_draw(&mut self, cmd: DrawCmd) {
        let _ = self.do_command(Command::Draw(cmd));
    }

    pub(crate) fn do_screen(&mut self, cmd: ScreenCmd) {
        let _ = self.do_command(Command::Screen(cmd));
    }

    pub(crate) fn do_input(&mut self, cmd: InputCmd) {
        let _ = self.do_command(Command::Input(cmd));
    }

    pub(crate) fn do_data(&mut self, cmd: DataCmd) -> Response {
        self.do_command(Command::Data(cmd))
    }

    pub(crate) fn do_hatch(&mut self) -> Turtle {
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

    fn do_command(&mut self, cmd: Command) -> Response {
        if self.issue_command.send(self.req(cmd)).is_ok() {
            if let Ok(result) = self.command_complete.recv() {
                return result;
            }
        }

        /* main thread has gone away; wait here to meet our doom */
        loop {
            std::thread::park();
        }
    }
}

#[derive(Default)]
enum Progression {
    #[default]
    Forward,
    Reverse,
}

#[derive(Default)]
pub(crate) struct TurtleData {
    cmds: Vec<DrawCmd>,               // already-drawn elements
    queue: VecDeque<DrawRequest>,     // new elements to draw
    current_command: Option<DrawCmd>, // what we're drawing now
    current_turtle_id: u64,           // which thread to notify on completion
    // _turtle_id: u64,               // TODO: doesn't the turtle need to know its own ID?
    percent: f64,
    progression: Progression,
    pos: Vec2d<isize>,
    angle: f64,
    insert_fill: Option<usize>,
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
    drawing_done: bool,
    speed: TurtleSpeed,
    turtle_shape: TurtlePolygon,
    last_point: Option<Vec2d<isize>>,
    poly: Vec<[f32; 2]>,
}

impl TurtleData {
    fn draw(&mut self, ds: &mut TurtleDrawState) {
        let mut index = 0;
        let mut done = false;
        // draw all the user commands
        while !done {
            ds.start_deg = ds.deg;
            let cmd = if index < self.cmds.len() {
                index += 1;
                let cmd = &self.cmds[index - 1];
                ds.deg += cmd.get_rotation(ds) % 360.;
                if ds.deg < 0. {
                    ds.deg += 360.;
                }
                Some(cmd)
            } else {
                ds.pct = self.percent.min(1.);
                done = true;
                self.current_command.as_ref()
            };

            let Some(cmd) = cmd else {
                break;
            };

            cmd.draw(ds);
        }

        // draw the turtle shape
        self.turtle_shape.draw(&crate::BLACK, ds.transform, ds);

        // save last known position and angle
        self.pos = [
            (ds.transform[0][2] * ds.size[0] / 2.) as isize,
            (ds.transform[1][2] * ds.size[1] / 2.) as isize,
        ];

        self.angle = if ds.deg + 90. >= 360. {
            ds.deg - 270.
        } else {
            ds.deg + 90.
        };
    }

    fn update(&mut self, delta_t: f64) {
        let s = self.speed.get();

        self.drawing_done = s == 0
            || match self.progression {
                Progression::Forward => self.percent >= 1.,
                Progression::Reverse => self.percent <= 0.,
            }
            || matches!(self.current_command, Some(DrawCmd::InstantaneousDraw(_)));
        if !self.drawing_done {
            let multiplier = s as f64 * 2.;

            match self.progression {
                Progression::Forward => self.percent += delta_t * multiplier,
                Progression::Reverse => self.percent -= delta_t * multiplier,
            }
        }

        if self.drawing_done && self.current_command.is_some() {
            self.drawing_done = false;
            if let Some(p) = self.last_point {
                if p != self.pos {
                    let new_point = [self.pos[0] as f32, self.pos[1] as f32];
                    self.poly.push(new_point);
                    self.last_point = Some(self.pos);
                }
            }
            let cmd = self.current_command.take().unwrap();
            if !matches!(cmd, DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Undo))
                && matches!(self.progression, Progression::Forward)
            {
                self.cmds.push(cmd.clone());
            }

            if matches!(
                cmd,
                DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon)
            ) {
                self.insert_fill = Some(self.cmds.len() - 1);
            }

            let _ = self.responder[&self.current_turtle_id].send(if cmd.is_stamp() {
                Response::StampID(self.cmds.len() - 1)
            } else {
                Response::Done
            });
        }

        if self.current_command.is_none() && !self.queue.is_empty() {
            let DrawRequest { cmd, turtle_id } = self.queue.pop_front().unwrap();
            self.current_turtle_id = turtle_id;

            if matches!(cmd, DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Undo)) {
                self.current_command = self.cmds.pop();
                self.progression = Progression::Reverse;
                self.percent = 1.;
            } else {
                self.current_command = Some(cmd);
                self.progression = Progression::Forward;
                self.percent = 0.;
            }
        }
    }
}

struct TurtleTask {
    gl: GlGraphics, // OpenGL drawing backend.
    window: GlutinWindow,
    turtle_num: u64,
    issue_command: Sender<Request>,
    receive_command: Receiver<Request>,
    bgcolor: types::Color,
    data: Vec<TurtleData>,
}

impl TurtleTask {
    fn run<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let mut turtle = self.spawn_turtle(0);
        let _ = std::thread::spawn(move || func(&mut turtle));

        let mut events = Events::new(EventSettings::new());

        while let Some(e) = events.next(&mut self.window) {
            while let Ok(req) = self.receive_command.try_recv() {
                let tid = req.turtle_id;
                let mut found = None;
                for (index, tdata) in self.data.iter().enumerate() {
                    if tdata.responder.contains_key(&tid) {
                        found = Some(index);
                        break;
                    }
                }
                if let Some(index) = found {
                    self.handle_command(index, req);
                }
            }

            if let Some(args) = e.render_args() {
                self.render(&args);
            }

            if let Some(args) = e.update_args() {
                self.update(&args);
            }

            if let Some(args) = e.button_args() {
                self.button(&args);
            }
        }
    }

    pub(crate) fn hatch_turtle(&mut self) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.turtle_num += 1;
        let newid = self.turtle_num;

        let mut td = TurtleData {
            percent: 2.,
            ..TurtleData::default()
        };
        td.responder.insert(newid, finished);
        self.data.push(td);

        Turtle::init(self.issue_command.clone(), command_complete, newid)
    }

    fn spawn_turtle(&mut self, which: usize) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.turtle_num += 1;
        let newid = self.turtle_num;
        self.data[which].responder.insert(newid, finished);

        Turtle::init(self.issue_command.clone(), command_complete, newid)
    }

    fn screen_cmd(&mut self, which: usize, cmd: ScreenCmd, turtle_id: u64) {
        let resp = self.data[which].responder.get(&turtle_id).unwrap().clone();
        match cmd {
            ScreenCmd::Speed(s) => {
                self.data[which].speed = s;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginFill => {
                self.data[which].last_point = Some(self.data[which].pos);
                self.data[which].poly = vec![[
                    self.data[which].pos[0] as f32,
                    self.data[which].pos[1] as f32,
                ]];
                self.data[which].queue.push_back(DrawRequest {
                    cmd: DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon),
                    turtle_id,
                });
            }
            ScreenCmd::EndFill => {
                if let Some(index) = self.data[which].insert_fill.take() {
                    let polygon = TurtlePolygon::new(&self.data[which].poly);
                    self.data[which].cmds[index] =
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Fill(polygon));
                    self.data[which].last_point = None;
                }
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {}
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.bgcolor = [r, g, b, 1.];
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data[which].cmds.clear();
                self.bgcolor = crate::BLACK;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data[which].cmds.len()
                    && matches!(
                        self.data[which].cmds[id],
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(true))
                    )
                {
                    self.data[which].cmds[id] =
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(false))
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
            ClearDirection::Forward => Either::Right(self.data[which].cmds.iter_mut()),
            ClearDirection::Reverse => Either::Left(self.data[which].cmds.iter_mut().rev()),
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if cmd.is_stamp() {
                    count -= 1;
                    *cmd = DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(false));
                }
            } else {
                break;
            }
        }
    }

    fn input_cmd(&mut self, which: usize, cmd: InputCmd, turtle_id: u64) {
        let resp = self.data[which].responder.get(&turtle_id).unwrap().clone();
        match cmd {
            InputCmd::OnKeyPress(f, k) => {
                self.data[which].onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn data_cmd(&mut self, which: usize, cmd: DataCmd, turtle_id: u64) {
        let resp = self.data[which].responder.get(&turtle_id).unwrap();
        let _ = match cmd {
            DataCmd::UndoBufferEntries => resp.send(Response::Count(self.data[which].cmds.len())),
            DataCmd::Towards(xpos, ypos) => {
                let curpos = [
                    self.data[which].pos[0] as f64,
                    self.data[which].pos[1] as f64,
                ];
                let x = xpos - curpos[0];
                let y = ypos - curpos[1];

                if x == 0. {
                    resp.send(Response::Heading(0.))
                } else {
                    resp.send(Response::Heading(
                        x.atan2(y) * 360. / (2.0 * std::f64::consts::PI),
                    ))
                }
            }
            DataCmd::Position => resp.send(Response::Position(self.data[which].pos)),
            DataCmd::Heading => resp.send(Response::Heading(self.data[which].angle)),
            DataCmd::Stamp => {
                self.data[which].queue.push_back(DrawRequest {
                    cmd: DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(true)),
                    turtle_id,
                });
                Ok(())
            }
        };
    }

    fn handle_command(&mut self, which: usize, req: Request) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(which, cmd, req.turtle_id),
            Command::Draw(cmd) => self.data[which].queue.push_back(DrawRequest {
                cmd,
                turtle_id: req.turtle_id,
            }),
            Command::Input(cmd) => self.input_cmd(which, cmd, req.turtle_id),
            Command::Data(cmd) => self.data_cmd(which, cmd, req.turtle_id),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle();
                let resp = &self.data[which].responder[&req.turtle_id];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |context, gl| {
            clear(self.bgcolor, gl);

            for turtle in self.data.iter_mut() {
                let mut ds = TurtleDrawState::new(
                    args.window_size,
                    context,
                    gl,
                    turtle.turtle_shape.clone(), // XXX: fix this?
                );
                turtle.draw(&mut ds);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for turtle in self.data.iter_mut() {
            turtle.update(args.dt);
        }
    }

    fn button(&mut self, args: &ButtonArgs) {
        let mut work = Vec::new();
        for (idx, turtle) in self.data.iter().enumerate() {
            match args {
                ButtonArgs {
                    state: ButtonState::Press,
                    button: Button::Keyboard(key),
                    ..
                } => {
                    if let Some(func) = turtle.onkeypress.get(key).copied() {
                        work.push((idx, func, *key));
                    }
                }
                ButtonArgs { state, button, .. } => {
                    println!("state={state:?}, button={button:?}");
                }
            }
        }
        for (idx, func, key) in work {
            let mut turtle = self.spawn_turtle(idx);
            let _ = std::thread::spawn(move || func(&mut turtle, key));
        }
    }
}
