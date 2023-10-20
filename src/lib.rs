use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender},
};

use command::{Command, DataCmd, DrawCmd, InputCmd, ScreenCmd};
use glutin_window::GlutinWindow;
use graphics::types::Vec2d;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, ButtonArgs, ButtonEvent, ButtonState, EventSettings, Events, Key, RenderArgs,
    RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};
pub use turtle::{Turtle, TurtleArgs};

mod command;
mod draw;
mod input;
pub mod turtle;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

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
    is_pen_down: bool,
    pos: Vec2d<isize>,
    angle: f64,
    size: Vec2d<f64>,
    bgcolor: [f32; 4],
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
}

pub enum Response {
    Done,
    Heading(f64),
    Position(Vec2d<isize>),
}

// transform is [[f64; 3]; 2]
// which looks like
// | a b c |
// | d e f |
//
// to transform (x, y), multiply by the transform like this
// | a*x + b*y + c| -> new x coordinate
// | d*x + e*y + f| -> new y coordinate
//
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
                is_pen_down: true,
                size: [xsize, ysize],
                bgcolor: WHITE,
                ..TurtleData::default()
            },
        };

        tt.run(func);
    }
}

impl TurtleTask {
    fn run<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let mut turtle = self.spawn_turtle();
        let _ = std::thread::spawn(move || func(&mut turtle));

        let mut events = Events::new(EventSettings::new());

        let mut command_complete = true;
        while let Some(e) = events.next(&mut self.window) {
            if let Ok(req) = self.receive_command.try_recv() {
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

            if !command_complete && self.data.current_command.is_none() {
                command_complete = true;
                let _ = self
                    .data
                    .responder
                    .get(&self.data.current_turtle_id)
                    .unwrap()
                    .send(Response::Done);
            }

            if command_complete && !self.data.queue.is_empty() {
                let DrawRequest { cmd, turtle_id } = self.data.queue.pop_front().unwrap();
                self.data.current_command = Some(cmd);
                self.data.current_turtle_id = turtle_id;
                self.data.percent = 0.;
                command_complete = false;
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
                self.data.bgcolor = BLACK;
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

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(self.data.bgcolor, gl);

            let mut transform = c.transform.trans(x, y).rot_deg(-90.);
            let mut pct = 1.;
            let mut full = false;
            let mut done = false;
            let mut deg: f64 = -90.;
            let mut pen_color = BLACK;
            let mut pen_width = 1.0;

            let mut index = 0;
            while !done {
                let cmd = if index < self.data.cmds.len() {
                    index += 1;
                    let cmd = self.data.cmds[index - 1];
                    deg += cmd.get_rotation() % 360.;
                    if deg < 0. {
                        deg += 360.;
                    }
                    Some(cmd)
                } else {
                    pct = self.data.percent.min(1.);
                    full = pct >= 1.;
                    done = true;
                    self.data.current_command
                };

                let Some(cmd) = cmd else {
                    break;
                };

                if full {
                    self.data.cmds.push(cmd);
                    self.data.current_command = None;
                }

                match cmd {
                    DrawCmd::Forward(dist) => {
                        if self.data.is_pen_down {
                            line_from_to(
                                pen_color,
                                pen_width,
                                [0., 0.],
                                [dist * pct, 0.],
                                transform,
                                gl,
                            );
                        }
                        transform = transform.trans(dist * pct, 0.);
                    }
                    DrawCmd::Right(deg) => transform = transform.rot_deg(deg * pct),
                    DrawCmd::Left(deg) => transform = transform.rot_deg(-deg * pct),
                    DrawCmd::PenDown => self.data.is_pen_down = true,
                    DrawCmd::PenUp => self.data.is_pen_down = false,
                    DrawCmd::GoTo(xpos, ypos) => {
                        transform = c.transform.trans(xpos + x, ypos + y).rot_deg(deg);
                    }
                    DrawCmd::PenColor(r, g, b) => {
                        pen_color = [r, g, b, 1.];
                    }
                    DrawCmd::PenWidth(width) => {
                        pen_width = width;
                    }
                }
            }

            self.data.pos = [
                (transform[0][2] * self.data.size[0] / 2.) as isize,
                (transform[1][2] * self.data.size[1] / 2.) as isize,
            ];
            self.data.angle = if deg + 90. >= 360. {
                deg - 270.
            } else {
                deg + 90.
            };

            let square = rectangle::square(0.0, 0.0, 10.0);
            ellipse(BLACK, square, transform.trans(-5., -5.), gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.data.percent < 1. {
            self.data.percent += args.dt * 60.;
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

pub struct Request {
    turtle_id: u64,
    cmd: Command,
}

struct DrawRequest {
    cmd: DrawCmd,
    turtle_id: u64,
}
