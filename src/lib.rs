use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Sender},
};

use glutin_window::GlutinWindow;
use graphics::types::Vec2d;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, ButtonArgs, ButtonEvent, ButtonState, EventSettings, Events, Key, RenderArgs,
    RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};
pub use turtle::Turtle;

mod draw;
mod input;
pub mod turtle;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

struct TurtleTask {
    gl: GlGraphics, // OpenGL drawing backend.
    cmds: Vec<Command>,
    queue: VecDeque<Request>,
    current_command: Option<Command>,
    current_turtle: u64,
    percent: f64,
    is_pen_down: bool,
    pos: Vec2d<isize>,
    angle: f64,
    size: Vec2d<f64>,
    bgcolor: [f32; 4],
    responder: HashMap<u64, Sender<Response>>,
    onkey: HashMap<Key, fn(&mut Turtle, Key)>,
}

pub enum Response {
    Done,
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
    pub fn start<F: FnOnce(&mut Turtle) + Send + 'static, S: Into<f64>>(
        xsize: S,
        ysize: S,
        func: F,
    ) {
        let (issue_command, receive_command) = mpsc::channel();
        let (finished, command_complete) = mpsc::channel();
        let xsize: f64 = xsize.into();
        let ysize: f64 = ysize.into();
        let mut turtle_id = 0;

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create a Glutin window.
        let mut window: GlutinWindow = WindowSettings::new("spinning-square", [xsize, ysize])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let mut tt = TurtleTask {
            gl: GlGraphics::new(opengl),
            cmds: Vec::new(),
            queue: VecDeque::new(),
            current_command: None,
            current_turtle: 0,
            percent: 0.,
            is_pen_down: true,
            pos: [0, 0],
            angle: 0.,
            size: [xsize, ysize],
            bgcolor: WHITE,
            responder: HashMap::new(),
            onkey: HashMap::new(),
        };

        turtle_id += 1;
        let mut turtle = Turtle::new(issue_command.clone(), command_complete, turtle_id);
        tt.responder.insert(turtle_id, finished);

        let _ = std::thread::spawn(move || func(&mut turtle));

        let mut events = Events::new(EventSettings::new());
        let mut command_complete = true;
        while let Some(e) = events.next(&mut window) {
            if let Ok(req) = receive_command.try_recv() {
                let resp = tt.responder.get(&req.turtle_id).unwrap();
                match req.cmd {
                    Command::Background(r, g, b) => {
                        tt.bgcolor = [r, g, b, 1.];
                        let _ = resp.send(Response::Done);
                    }
                    Command::ClearScreen => {
                        tt.cmds.clear();
                        tt.bgcolor = BLACK;
                        let _ = resp.send(Response::Done);
                    }
                    Command::OnKey(f, k) => {
                        tt.onkey.insert(k, f);
                        let _ = resp.send(Response::Done);
                    }
                    _ => {
                        tt.queue.push_back(req);
                    }
                }
            }

            if !command_complete && tt.current_command.is_none() {
                command_complete = true;
                let _ = tt
                    .responder
                    .get(&tt.current_turtle)
                    .unwrap()
                    .send(Response::Done);
            }

            if command_complete && !tt.queue.is_empty() {
                let Request { turtle_id, cmd } = tt.queue.pop_front().unwrap();
                tt.current_command = Some(cmd);
                tt.current_turtle = turtle_id;
                tt.percent = 0.;
                command_complete = false;
            }

            if let Some(args) = e.render_args() {
                tt.render(&args);
            }

            if let Some(args) = e.update_args() {
                tt.update(&args);
            }

            if let Some(args) = e.button_args() {
                match args {
                    ButtonArgs { state, button, .. } => {
                        if let Button::Keyboard(key) = button {
                            let func = if state == ButtonState::Press {
                                tt.onkey.get_mut(&key)
                            } else {
                                None // tt.onkeyrelease.get_mut(&key)
                            };
                            if let Some(func) = func.cloned() {
                                let (finished, command_complete) = mpsc::channel();
                                turtle_id += 1;
                                let mut turtle =
                                    Turtle::new(issue_command.clone(), command_complete, turtle_id);
                                tt.responder.insert(turtle_id, finished);
                                let _ = std::thread::spawn(move || func(&mut turtle, key));
                            }
                        }
                    }
                }
            }
        }
    }
}

impl TurtleTask {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(self.bgcolor, gl);

            let mut transform = c.transform.trans(x, y).rot_deg(-90.);
            let mut pct = 1.;
            let mut full = false;
            let mut done = false;
            let mut deg: f64 = -90.;
            let mut pen_color = BLACK;
            let mut pen_width = 1.0;

            let mut index = 0;
            while !done {
                let cmd = if index < self.cmds.len() {
                    index += 1;
                    let cmd = self.cmds[index - 1];
                    deg += cmd.get_rotation() % 360.;
                    if deg < 0. {
                        deg += 360.;
                    }
                    Some(cmd)
                } else {
                    pct = self.percent.min(1.);
                    full = pct >= 1.;
                    done = true;
                    self.current_command
                };

                let Some(cmd) = cmd else {
                    break;
                };

                if full {
                    self.cmds.push(cmd);
                    self.current_command = None;
                }

                match cmd {
                    Command::Forward(dist) => {
                        if self.is_pen_down {
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
                    Command::Right(deg) => transform = transform.rot_deg(deg * pct),
                    Command::Left(deg) => transform = transform.rot_deg(-deg * pct),
                    Command::PenDown => self.is_pen_down = true,
                    Command::PenUp => self.is_pen_down = false,
                    Command::GoTo(xpos, ypos) => {
                        transform = c.transform.trans(xpos + x, ypos + y).rot_deg(deg);
                    }
                    Command::PenColor(r, g, b) => {
                        pen_color = [r, g, b, 1.];
                    }
                    Command::PenWidth(width) => {
                        pen_width = width;
                    }
                    Command::OnKey(_, _) | Command::Background(_, _, _) | Command::ClearScreen => {
                        panic!("{cmd:?} is not a drawing command")
                    }
                }
            }

            self.pos = [
                (transform[0][2] * self.size[0] / 2.) as isize,
                (transform[1][2] * self.size[1] / 2.) as isize,
            ];
            self.angle = if deg + 90. >= 360. {
                deg - 270.
            } else {
                deg + 90.
            };

            let square = rectangle::square(0.0, 0.0, 10.0);
            ellipse(BLACK, square, transform.trans(-5., -5.), gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.percent < 1. {
            self.percent += args.dt * 60.;
        }
    }
}

pub struct Request {
    turtle_id: u64,
    cmd: Command,
}

#[derive(Copy, Clone, Debug)]
pub enum Command {
    Forward(f64),
    Right(f64),
    Left(f64),
    PenDown,
    PenUp,
    GoTo(f64, f64),
    ClearScreen,
    PenColor(f32, f32, f32),
    PenWidth(f64),
    Background(f32, f32, f32),
    OnKey(fn(&mut Turtle, Key), Key),
}

impl Command {
    fn get_rotation(&self) -> f64 {
        match self {
            Command::Right(deg) => *deg,
            Command::Left(deg) => -*deg,
            _ => 0.,
        }
    }
}
