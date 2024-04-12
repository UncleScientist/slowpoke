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
            bgcolor: crate::WHITE,
            data: vec![TurtleData {
                size: [xsize, ysize],
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
    turtle_id: u64,
    percent: f64,
    progression: Progression,
    pos: Vec2d<isize>,
    angle: f64,
    size: Vec2d<f64>,
    insert_fill: Option<usize>,
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
    drawing_done: bool,
    speed: TurtleSpeed,
    pub(crate) turtle_shape: TurtlePolygon,
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
            (ds.transform[0][2] * self.size[0] / 2.) as isize,
            (ds.transform[1][2] * self.size[1] / 2.) as isize,
        ];

        self.angle = if ds.deg + 90. >= 360. {
            ds.deg - 270.
        } else {
            ds.deg + 90.
        };
    }
}

struct TurtleTask {
    gl: GlGraphics, // OpenGL drawing backend.
    window: GlutinWindow,
    issue_command: Sender<Request>,
    receive_command: Receiver<Request>,
    bgcolor: types::Color,
    data: Vec<TurtleData>,
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
                self.data[0].size = args.window_size;
            }
        }
    }

    fn spawn_turtle(&mut self) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.data[0].turtle_id += 1;
        let newid = self.data[0].turtle_id;
        self.data[0].responder.insert(newid, finished);

        Turtle::init(
            self.issue_command.clone(),
            command_complete,
            self.data[0].turtle_id,
        )
    }

    fn screen_cmd(&mut self, cmd: ScreenCmd, turtle_id: u64) {
        let resp = self.data[0].responder.get(&turtle_id).unwrap().clone();
        match cmd {
            ScreenCmd::Speed(s) => {
                self.data[0].speed = s;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginFill => {
                self.data[0].last_point = Some(self.data[0].pos);
                self.data[0].poly = vec![[self.data[0].pos[0] as f32, self.data[0].pos[1] as f32]];
                self.data[0].queue.push_back(DrawRequest {
                    cmd: DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon),
                    turtle_id,
                });
            }
            ScreenCmd::EndFill => {
                if let Some(index) = self.data[0].insert_fill.take() {
                    let polygon = TurtlePolygon::new(&self.data[0].poly);
                    self.data[0].cmds[index] =
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Fill(polygon));
                    self.data[0].last_point = None;
                }
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {}
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.bgcolor = [r, g, b, 1.];
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data[0].cmds.clear();
                self.bgcolor = crate::BLACK;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data[0].cmds.len()
                    && matches!(
                        self.data[0].cmds[id],
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(true))
                    )
                {
                    self.data[0].cmds[id] =
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(false))
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
            ClearDirection::Forward => Either::Right(self.data[0].cmds.iter_mut()),
            ClearDirection::Reverse => Either::Left(self.data[0].cmds.iter_mut().rev()),
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

    fn input_cmd(&mut self, cmd: InputCmd, turtle_id: u64) {
        let resp = self.data[0].responder.get(&turtle_id).unwrap().clone();
        match cmd {
            InputCmd::OnKeyPress(f, k) => {
                self.data[0].onkeypress.insert(k, f);
                let _ = resp.send(Response::Done);
            }
        }
    }

    fn data_cmd(&mut self, cmd: DataCmd, turtle_id: u64) {
        let resp = self.data[0].responder.get(&turtle_id).unwrap();
        let _ = match cmd {
            DataCmd::Position => resp.send(Response::Position(self.data[0].pos)),
            DataCmd::Heading => resp.send(Response::Heading(self.data[0].angle)),
            DataCmd::Stamp => {
                self.data[0].queue.push_back(DrawRequest {
                    cmd: DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Stamp(true)),
                    turtle_id,
                });
                Ok(())
            }
        };
    }

    fn handle_command(&mut self, req: Request) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(cmd, req.turtle_id),
            Command::Draw(cmd) => self.data[0].queue.push_back(DrawRequest {
                cmd,
                turtle_id: req.turtle_id,
            }),
            Command::Input(cmd) => self.input_cmd(cmd, req.turtle_id),
            Command::Data(cmd) => self.data_cmd(cmd, req.turtle_id),
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        self.gl.draw(args.viewport(), |context, gl| {
            // Clear the screen.
            clear(self.bgcolor, gl);
            for turtle in self.data.iter_mut() {
                let mut ds = TurtleDrawState::new(
                    args.window_size,
                    context,
                    gl,
                    turtle.turtle_shape.clone(),
                );
                turtle.draw(&mut ds);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let s = self.data[0].speed.get();

        self.data[0].drawing_done = s == 0
            || match self.data[0].progression {
                Progression::Forward => self.data[0].percent >= 1.,
                Progression::Reverse => self.data[0].percent <= 0.,
            }
            || matches!(
                self.data[0].current_command,
                Some(DrawCmd::InstantaneousDraw(_))
            );
        if !self.data[0].drawing_done {
            let multiplier = s as f64 * 2.;

            match self.data[0].progression {
                Progression::Forward => self.data[0].percent += args.dt * multiplier,
                Progression::Reverse => self.data[0].percent -= args.dt * multiplier,
            }
        }

        if self.data[0].drawing_done && self.data[0].current_command.is_some() {
            self.data[0].drawing_done = false;
            if let Some(p) = self.data[0].last_point {
                if p != self.data[0].pos {
                    let new_point = [self.data[0].pos[0] as f32, self.data[0].pos[1] as f32];
                    self.data[0].poly.push(new_point);
                    self.data[0].last_point = Some(self.data[0].pos);
                }
            }
            let cmd = self.data[0].current_command.take().unwrap();
            if !matches!(cmd, DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Undo))
                && matches!(self.data[0].progression, Progression::Forward)
            {
                self.data[0].cmds.push(cmd.clone());
            }

            if matches!(
                cmd,
                DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon)
            ) {
                self.data[0].insert_fill = Some(self.data[0].cmds.len() - 1);
            }

            let _ =
                self.data[0].responder[&self.data[0].current_turtle_id].send(if cmd.is_stamp() {
                    Response::StampID(self.data[0].cmds.len() - 1)
                } else {
                    Response::Done
                });
        }

        if self.data[0].current_command.is_none() && !self.data[0].queue.is_empty() {
            let DrawRequest { cmd, turtle_id } = self.data[0].queue.pop_front().unwrap();
            self.data[0].current_turtle_id = turtle_id;

            if matches!(cmd, DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Undo)) {
                self.data[0].current_command = self.data[0].cmds.pop();
                self.data[0].progression = Progression::Reverse;
                self.data[0].percent = 1.;
            } else {
                self.data[0].current_command = Some(cmd);
                self.data[0].progression = Progression::Forward;
                self.data[0].percent = 0.;
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
                if let Some(func) = self.data[0].onkeypress.get(key).copied() {
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
