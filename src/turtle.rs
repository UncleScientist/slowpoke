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
    RenderEvent, ResizeEvent, UpdateArgs, UpdateEvent, WindowSettings,
};

use crate::{
    color_names::TurtleColor,
    command::{Command, DataCmd, InputCmd, ScreenCmd, TurtleDrawState},
    polygon::TurtlePolygon,
    speed::TurtleSpeed,
    DrawCmd, Request, Response,
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

        let turtle_shape = [[0., 0.], [-15., 6.], [-10., 0.], [-15., -6.], [0., 0.]];

        let (issue_command, receive_command) = mpsc::channel();
        let mut tt = TurtleTask {
            gl: GlGraphics::new(opengl),
            window,
            issue_command,
            receive_command,
            data: TurtleData {
                size: [xsize, ysize],
                bgcolor: crate::WHITE,
                percent: 2.,
                ..TurtleData::default()
            },
            turtle_shape: TurtlePolygon::new(&turtle_shape),
            shape_offset: (-0., -0.),
            last_point: None,
            poly: Vec::new(),
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

struct TurtleTask {
    gl: GlGraphics, // OpenGL drawing backend.
    window: GlutinWindow,
    issue_command: Sender<Request>,
    receive_command: Receiver<Request>,
    data: TurtleData,
    turtle_shape: TurtlePolygon,
    shape_offset: (f64, f64),
    last_point: Option<Vec2d<isize>>,
    poly: Vec<[f32; 2]>,
}

#[derive(Default)]
enum Progression {
    #[default]
    Forward,
    Reverse,
}

#[derive(Default)]
struct TurtleData {
    cmds: Vec<DrawCmd>,               // already-drawn elements
    queue: VecDeque<DrawRequest>,     // new elements to draw
    current_command: Option<DrawCmd>, // what we're drawing now
    current_turtle_id: u64,           // which thread to notify on completion
    turtle_id: u64,
    percent: f64,
    progression: Progression,
    pos: Vec2d<isize>,
    angle: f64,
    size: Vec2d<f64>,
    bgcolor: types::Color,
    insert_fill: Option<usize>,
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
    drawing_done: bool,
    speed: TurtleSpeed,
}

impl TurtleTask {
    fn run<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let mut turtle = self.spawn_turtle();
        let _ = std::thread::spawn(move || func(&mut turtle));

        let mut events = Events::new(EventSettings::new());

        while let Some(e) = events.next(&mut self.window) {
            while let Ok(req) = self.receive_command.try_recv() {
                self.handle_command(req);
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

            if let Some(args) = e.resize_args() {
                self.data.size = args.window_size;
            }
        }
    }

    fn spawn_turtle(&mut self) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.data.turtle_id += 1;
        self.data.responder.insert(self.data.turtle_id, finished);

        Turtle::init(
            self.issue_command.clone(),
            command_complete,
            self.data.turtle_id,
        )
    }

    fn screen_cmd(&mut self, cmd: ScreenCmd, turtle_id: u64) {
        let resp = self.data.responder.get(&turtle_id).unwrap().clone();
        match cmd {
            ScreenCmd::Speed(s) => {
                self.data.speed = s;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginFill => {
                self.last_point = Some(self.data.pos);
                self.poly = vec![[self.data.pos[0] as f32, self.data.pos[1] as f32]];
                self.data.queue.push_back(DrawRequest {
                    cmd: DrawCmd::Skip,
                    turtle_id,
                });
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::EndFill => {
                if let Some(index) = self.data.insert_fill.take() {
                    let polygon = TurtlePolygon::new(&self.poly);
                    self.data.cmds[index] = DrawCmd::Fill(polygon);
                    self.last_point = None;
                }
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {}
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.data.bgcolor = [r, g, b, 1.];
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data.cmds.clear();
                self.data.bgcolor = crate::BLACK;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data.cmds.len() && matches!(self.data.cmds[id], DrawCmd::Stamp(true)) {
                    self.data.cmds[id] = DrawCmd::Stamp(false);
                }
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamps(count) => {
                #[allow(clippy::comparison_chain)]
                if count < 0 {
                    self.clear_stamps(-count, ClearDirection::Reverse);
                } else if count == 0 {
                    self.clear_stamps(isize::MAX, ClearDirection::Forward);
                } else {
                    self.clear_stamps(count, ClearDirection::Forward);
                }
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn clear_stamps(&mut self, mut count: isize, dir: ClearDirection) {
        let mut iter = match dir {
            ClearDirection::Forward => Either::Right(self.data.cmds.iter_mut()),
            ClearDirection::Reverse => Either::Left(self.data.cmds.iter_mut().rev()),
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if matches!(cmd, DrawCmd::Stamp(true)) {
                    count -= 1;
                    *cmd = DrawCmd::Stamp(false);
                }
            } else {
                break;
            }
        }
    }

    fn input_cmd(&mut self, cmd: InputCmd, turtle_id: u64) {
        let resp = self.data.responder.get(&turtle_id).unwrap();
        match cmd {
            InputCmd::OnKeyPress(f, k) => {
                self.data.onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn data_cmd(&mut self, cmd: DataCmd, turtle_id: u64) {
        let resp = self.data.responder.get(&turtle_id).unwrap();
        let _ = match cmd {
            DataCmd::Position => resp.send(Response::Position(self.data.pos)),
            DataCmd::Heading => resp.send(Response::Heading(self.data.angle)),
            DataCmd::Stamp => {
                self.data.queue.push_back(DrawRequest {
                    cmd: DrawCmd::Stamp(true),
                    turtle_id,
                });
                Ok(())
            }
        };
    }

    fn handle_command(&mut self, req: Request) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(cmd, req.turtle_id),
            Command::Draw(cmd) => self.data.queue.push_back(DrawRequest {
                cmd,
                turtle_id: req.turtle_id,
            }),
            Command::Input(cmd) => self.input_cmd(cmd, req.turtle_id),
            Command::Data(cmd) => self.data_cmd(cmd, req.turtle_id),
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |context, gl| {
            // Clear the screen.
            clear(self.data.bgcolor, gl);
            let win_center = context.transform.trans(x, y);

            let mut ds = TurtleDrawState {
                context,
                x,
                y,
                size: args.window_size,
                is_pen_down: true,
                transform: win_center,
                win_center,
                pct: 1.,
                deg: -0.,
                start_deg: -0.,
                pen_color: crate::BLACK,
                fill_color: [0.4, 0.5, 0.6, 1.0],
                pen_width: 0.5,
                gl,
                shape: self.turtle_shape.clone(),
                shape_offset: self.shape_offset,
            };

            let mut index = 0;
            let mut done = false;
            while !done {
                ds.start_deg = ds.deg;
                let cmd = if index < self.data.cmds.len() {
                    index += 1;
                    let cmd = &self.data.cmds[index - 1];
                    ds.deg += cmd.get_rotation(&ds) % 360.;
                    if ds.deg < 0. {
                        ds.deg += 360.;
                    }
                    Some(cmd)
                } else {
                    ds.pct = self.data.percent.min(1.);
                    done = true;
                    self.data.current_command.as_ref()
                };

                let Some(cmd) = cmd else {
                    break;
                };

                cmd.draw(&mut ds);
            }

            self.data.pos = [
                (ds.transform[0][2] * self.data.size[0] / 2.) as isize,
                (ds.transform[1][2] * self.data.size[1] / 2.) as isize,
            ];
            self.data.angle = if ds.deg + 90. >= 360. {
                ds.deg - 270.
            } else {
                ds.deg + 90.
            };

            self.data.drawing_done = match self.data.progression {
                Progression::Forward => ds.pct >= 1.,
                Progression::Reverse => ds.pct <= 0.,
            };

            let transform = ds.transform.trans(self.shape_offset.0, self.shape_offset.1);
            self.turtle_shape.draw(&crate::BLACK, &transform, &mut ds);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let s = self.data.speed.get();
        if s == 0 {
            self.data.drawing_done = true;
        } else if self.data.percent >= 0. && self.data.percent <= 1. {
            let multiplier = (11 - s) as f64 * 2.;

            match self.data.progression {
                Progression::Forward => self.data.percent += args.dt * multiplier,
                Progression::Reverse => self.data.percent -= args.dt * multiplier,
            }
        }

        if self.data.drawing_done && self.data.current_command.is_some() {
            self.data.drawing_done = false;
            if let Some(p) = self.last_point {
                if p != self.data.pos {
                    self.poly
                        .push([self.data.pos[0] as f32, self.data.pos[1] as f32]);
                    self.last_point = Some(self.data.pos);
                }
            }
            let cmd = self.data.current_command.take().unwrap();
            if !matches!(cmd, DrawCmd::Undo)
                && matches!(self.data.progression, Progression::Forward)
            {
                self.data.cmds.push(cmd.clone());
            }
            self.data.current_command = None; // TODO: clean this up

            if matches!(cmd, DrawCmd::Skip) {
                self.data.insert_fill = Some(self.data.cmds.len() - 1);
            }

            let _ = self
                .data
                .responder
                .get(&self.data.current_turtle_id)
                .unwrap()
                .send(if matches!(cmd, DrawCmd::Stamp(_)) {
                    Response::StampID(self.data.cmds.len() - 1)
                } else {
                    Response::Done
                });
        }

        if self.data.current_command.is_none() && !self.data.queue.is_empty() {
            let DrawRequest { cmd, turtle_id } = self.data.queue.pop_front().unwrap();
            self.data.current_turtle_id = turtle_id;

            if matches!(cmd, DrawCmd::Undo) {
                self.data.current_command = self.data.cmds.pop();
                self.data.progression = Progression::Reverse;
                self.data.percent = 1.;
            } else {
                self.data.current_command = Some(cmd);
                self.data.progression = Progression::Forward;
                self.data.percent = 0.;
            }
        }
    }

    fn button(&mut self, args: &ButtonArgs) {
        match args {
            ButtonArgs {
                state: ButtonState::Press,
                button: Button::Keyboard(key),
                ..
            } => {
                if let Some(func) = self.data.onkeypress.get(key).copied() {
                    let mut turtle = self.spawn_turtle();
                    let key = *key;
                    let _ = std::thread::spawn(move || func(&mut turtle, key));
                }
            }
            ButtonArgs { state, button, .. } => {
                println!("state={state:?}, button={button:?}");
            }
        }
    }
}
