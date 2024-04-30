use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{self, Receiver, Sender},
};

use either::Either;
use glutin_window::GlutinWindow;
use graphics::{
    math::identity,
    types::{self, Vec2d},
};
use graphics::{Context, Transformed};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    Button, ButtonArgs, ButtonEvent, ButtonState, EventSettings, Events, Key, RenderArgs,
    RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};

use crate::{
    color_names::TurtleColor,
    command::{
        Command, DataCmd, DrawCmd, InputCmd, InstantaneousDrawCmd, MotionCmd, RotateCmd, ScreenCmd,
        TimedDrawCmd,
    },
    polygon::{generate_default_shapes, TurtlePolygon, TurtleShape},
    speed::TurtleSpeed,
    Request, Response, TurtleShapeName,
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
            shapes: generate_default_shapes(),
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
struct PolygonBuilder {
    last_point: Option<Vec2d<isize>>,
    verticies: Vec<[f32; 2]>,
}

impl PolygonBuilder {
    fn start(&mut self, pos: Vec2d<isize>) {
        self.last_point = Some(pos);
        self.verticies = vec![[pos[0] as f32, pos[1] as f32]];
    }

    fn update(&mut self, pos: Vec2d<isize>) {
        if let Some(p) = self.last_point {
            if p != pos {
                let new_point = [pos[0] as f32, pos[1] as f32];
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

// To calculate points as we go along:
// For Lines:
//  - pen color / pen width of each line segment
//  - start & end of each line segment
//
// For polygons:
//  - fill color
//  - tesselated verticies
//  - border color

#[derive(Debug)]
struct LineInfo {
    pen_color: TurtleColor,
    pen_width: f64,
    points: Vec<Vec2d<isize>>,
}

#[derive(Debug)]
struct PolyInfo {
    fill_color: [f32; 4],
    border_color: [f32; 4],
    poly: Vec<TurtlePolygon>,
}

#[derive(Debug)]
enum DrawInfo {
    Line(LineInfo),
    Poly(PolyInfo),
}

#[derive(Debug)]
struct CurrentShape {
    pen_color: TurtleColor,
    pen_width: f64,
    fill_color: TurtleColor,
    transform: [[f64; 3]; 2],
    points: Vec<Vec2d<isize>>,
    angle: f64,
}

trait TurtlePosition<T> {
    fn pos(&self) -> [T; 2];
}

impl TurtlePosition<f64> for CurrentShape {
    fn pos(&self) -> [f64; 2] {
        [self.transform[0][2] as f64, self.transform[1][2] as f64]
    }
}

impl TurtlePosition<isize> for CurrentShape {
    fn pos(&self) -> [isize; 2] {
        [self.transform[0][2] as isize, self.transform[1][2] as isize]
    }
}

impl Default for CurrentShape {
    fn default() -> Self {
        Self {
            pen_color: "black".into(),
            pen_width: 0.5,
            fill_color: "black".into(),
            transform: identity(),
            points: Vec::new(),
            angle: 0.,
        }
    }
}

impl CurrentShape {
    fn angle(&self) -> f64 {
        self.angle
    }

    fn save_point(&mut self) {
        let x = self.transform[0][2].round() as isize;
        let y = self.transform[1][2].round() as isize;
        self.points.push([x, y]);
    }

    fn apply(&mut self, cmd: &DrawCmd) -> Option<DrawInfo> {
        match cmd {
            DrawCmd::TimedDraw(td) => match td {
                TimedDrawCmd::Motion(motion) => {
                    if self.points.is_empty() {
                        self.save_point();
                    }
                    match motion {
                        MotionCmd::Forward(dist) => {
                            self.transform = self.transform.trans(*dist, 0.);
                        }
                        MotionCmd::Teleport(x, y) | MotionCmd::GoTo(x, y) => {
                            self.transform = identity().trans(*x, *y).rot_deg(self.angle);
                        }
                        MotionCmd::SetX(x) => {
                            let cur_y = self.transform[1][2];
                            self.transform = identity().trans(*x, cur_y).rot_deg(self.angle);
                        }
                        MotionCmd::SetY(y) => {
                            let cur_x = self.transform[0][2];
                            self.transform = identity().trans(cur_x, *y).rot_deg(self.angle);
                        }
                    }
                    self.save_point();
                }
                TimedDrawCmd::Rotate(rotation) => match rotation {
                    RotateCmd::Right(angle) => {
                        self.transform = self.transform.rot_deg(*angle);
                        self.angle += angle;
                    }
                    RotateCmd::Left(angle) => {
                        self.transform = self.transform.rot_deg(-*angle);
                        self.angle -= angle;
                    }
                    RotateCmd::SetHeading(h) => {
                        self.transform = self.transform.rot_deg(h - self.angle);
                        self.angle = *h;
                    }
                },
            },
            DrawCmd::InstantaneousDraw(id) => match id {
                InstantaneousDrawCmd::Undo => {}
                InstantaneousDrawCmd::BackfillPolygon => {}
                InstantaneousDrawCmd::PenDown => {}
                InstantaneousDrawCmd::PenUp => {}
                InstantaneousDrawCmd::PenColor(pc) => {
                    let info = self.generate_line_info();
                    self.pen_color = *pc;
                    return Some(info);
                }
                InstantaneousDrawCmd::FillColor(fc) => {
                    self.fill_color = *fc;
                }
                InstantaneousDrawCmd::PenWidth(pw) => {
                    let info = self.generate_line_info();
                    self.pen_width = *pw;
                    return Some(info);
                }
                InstantaneousDrawCmd::Dot(_, _) => {}
                InstantaneousDrawCmd::Stamp(_) => {}
                InstantaneousDrawCmd::Fill(_) => {}
            },
        }

        None
    }

    fn generate_line_info(&mut self) -> DrawInfo {
        let li = LineInfo {
            pen_color: self.pen_color,
            pen_width: self.pen_width,
            points: self.points.split_off(0),
        };
        DrawInfo::Line(li)
    }
}

#[derive(Default)]
pub(crate) struct TurtleData {
    cmds: Vec<DrawCmd>,               // already-drawn elements
    queue: VecDeque<DrawRequest>,     // new elements to draw
    current_command: Option<DrawCmd>, // what we're drawing now
    elements: Vec<DrawInfo>,
    current_shape: CurrentShape, // per-segment information

    current_turtle_id: u64, // which thread to notify on completion
    // _turtle_id: u64,               // TODO: doesn't the turtle need to know its own ID?
    percent: f64,
    progression: Progression,
    insert_fill: Option<usize>,
    responder: HashMap<u64, Sender<Response>>,
    onkeypress: HashMap<Key, fn(&mut Turtle, Key)>,
    drawing_done: bool,
    speed: TurtleSpeed,
    turtle_shape: TurtleShape,
    fill_poly: PolygonBuilder,
    shape_poly: PolygonBuilder,
}

impl TurtleData {
    fn do_command(&mut self, cmd: &DrawCmd) {
        if let Some(info) = self.current_shape.apply(cmd) {
            self.elements.push(info);
        }
    }

    fn draw(&self, context: &Context, gl: &mut GlGraphics) {
        for element in &self.elements {
            match element {
                DrawInfo::Line(line) => {
                    for pair in line.points.as_slice().windows(2) {
                        graphics::line_from_to(
                            line.pen_color.into(),
                            line.pen_width,
                            [pair[0][0] as f64, pair[0][1] as f64],
                            [pair[1][0] as f64, pair[1][1] as f64],
                            context.transform,
                            gl,
                        );
                    }
                }
                DrawInfo::Poly(_) => todo!(),
            }
        }

        // draw the rest of the points
        for pair in self.current_shape.points.as_slice().windows(2) {
            graphics::line_from_to(
                self.current_shape.pen_color.into(),
                self.current_shape.pen_width,
                [pair[0][0] as f64, pair[0][1] as f64],
                [pair[1][0] as f64, pair[1][1] as f64],
                context.transform,
                gl,
            );
        }

        let trans = context
            .transform
            .trans(
                self.current_shape.transform[0][2],
                self.current_shape.transform[1][2],
            )
            .rot_deg(self.current_shape.angle);

        // draw the turtle
        self.turtle_shape.shape.draw(&crate::BLACK, trans, gl);
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
            self.fill_poly.update(self.current_shape.pos());
            self.shape_poly.update(self.current_shape.pos());

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

            self.do_command(&cmd);

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
    shapes: HashMap<String, TurtleShape>,
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
            ScreenCmd::BeginPoly => {
                let pos_copy = self.data[which].current_shape.pos();
                self.data[which].shape_poly.start(pos_copy);
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::EndPoly => {
                self.data[which].shape_poly.close();
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::BeginFill => {
                let pos_copy = self.data[which].current_shape.pos();
                self.data[which].fill_poly.start(pos_copy);
                self.data[which].queue.push_back(DrawRequest {
                    cmd: DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon),
                    turtle_id,
                });
            }
            ScreenCmd::EndFill => {
                if let Some(index) = self.data[which].insert_fill.take() {
                    let polygon = TurtlePolygon::new(&self.data[which].fill_poly.verticies);
                    self.data[which].cmds[index] =
                        DrawCmd::InstantaneousDraw(InstantaneousDrawCmd::Fill(polygon));
                    self.data[which].fill_poly.last_point = None;
                }
                let _ = resp.send(Response::Done);
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
        let resp = self.data[which].responder.get(&turtle_id).unwrap().clone();
        let _ = match cmd {
            DataCmd::GetPoly => resp.send(Response::Polygon(
                self.data[which].shape_poly.verticies.clone(),
            )),
            DataCmd::TurtleShape(shape) => {
                if let TurtleShapeName::Shape(name) = shape {
                    self.data[which].turtle_shape = self.shapes[&name].clone();
                }
                resp.send(Response::Name(self.data[which].turtle_shape.name.clone()))
            }
            DataCmd::UndoBufferEntries => resp.send(Response::Count(self.data[which].cmds.len())),
            DataCmd::Towards(xpos, ypos) => {
                let curpos: [f64; 2] = self.data[which].current_shape.pos();
                let x = xpos - curpos[0];
                let y = ypos + curpos[1];

                if x == 0. {
                    resp.send(Response::Heading(0.))
                } else {
                    resp.send(Response::Heading(
                        x.atan2(y) * 360. / (2.0 * std::f64::consts::PI),
                    ))
                }
            }
            DataCmd::Position => {
                resp.send(Response::Position(self.data[which].current_shape.pos()))
            }
            DataCmd::Heading => {
                resp.send(Response::Heading(self.data[which].current_shape.angle()))
            }
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
        self.gl.draw(args.viewport(), |context, gl| {
            graphics::clear(self.bgcolor, gl);

            let centered = context.trans(args.window_size[0] / 2., args.window_size[1] / 2.);

            for turtle in &self.data {
                turtle.draw(&centered, gl);
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
