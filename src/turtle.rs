use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender},
};

use glutin_window::GlutinWindow;
use graphics::types::{self, Vec2d};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, ButtonArgs, ButtonEvent, ButtonState, EventSettings, Events, Key, RenderArgs,
    RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};

use crate::{
    command::DataCmd,
    command::InputCmd,
    command::ScreenCmd,
    command::{Command, TurtleDrawState},
    DrawCmd, Request, Response,
};

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
            .build()
            .unwrap();

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
}

#[derive(Default)]
struct TurtleData {
    cmds: Vec<DrawCmd>,               // already-drawn elements
    queue: VecDeque<DrawRequest>,     // new elements to draw
    current_command: Option<DrawCmd>, // what we're drawing now
    current_turtle_id: u64,           // which thread to notify on completion
    turtle_id: u64,
    percent: f64,
    pos: Vec2d<isize>,
    angle: f64,
    size: Vec2d<f64>,
    bgcolor: types::Color,
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
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
        let resp = self.data.responder.get(&turtle_id).unwrap();
        match cmd {
            ScreenCmd::Background(r, g, b) => {
                self.data.bgcolor = [r, g, b, 1.];
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data.cmds.clear();
                self.data.bgcolor = crate::BLACK;
                let _ = resp.send(Response::Done);
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
        let _ = resp.send(match cmd {
            DataCmd::Position => Response::Position(self.data.pos),
            DataCmd::Heading => Response::Heading(self.data.angle),
        });
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

            let mut ds = TurtleDrawState {
                context,
                x,
                y,
                size: args.window_size,
                is_pen_down: true,
                transform: context.transform.trans(x, y).rot_deg(-90.),
                pct: 1.,
                deg: -90.,
                start_deg: -90.,
                pen_color: crate::BLACK,
                pen_width: 0.5,
                gl,
            };

            let mut index = 0;
            let mut done = false;
            while !done {
                ds.start_deg = ds.deg;
                let cmd = if index < self.data.cmds.len() {
                    index += 1;
                    let cmd = self.data.cmds[index - 1];
                    ds.deg += cmd.get_rotation(&ds) % 360.;
                    if ds.deg < 0. {
                        ds.deg += 360.;
                    }
                    Some(cmd)
                } else {
                    ds.pct = self.data.percent.min(1.);
                    done = true;
                    self.data.current_command
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

            let square = rectangle::square(0.0, 0.0, 10.0);
            ellipse(crate::BLACK, square, ds.transform.trans(-5., -5.), gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.data.percent < 1. {
            self.data.percent += args.dt * 10.; // TODO: make this smarter
        }

        if self.data.percent >= 1. && self.data.current_command.is_some() {
            self.data.cmds.push(self.data.current_command.unwrap());
            self.data.current_command = None; // TODO: clean this up

            let _ = self
                .data
                .responder
                .get(&self.data.current_turtle_id)
                .unwrap()
                .send(Response::Done);
        }

        if self.data.current_command.is_none() && !self.data.queue.is_empty() {
            let DrawRequest { cmd, turtle_id } = self.data.queue.pop_front().unwrap();
            self.data.current_command = Some(cmd);
            self.data.current_turtle_id = turtle_id;
            self.data.percent = 0.;
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
