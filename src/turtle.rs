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
        Command, DataCmd, DrawRequest, InputCmd, InstantaneousDrawCmd, ScreenCmd, TimedDrawCmd,
    },
    generate::{CurrentTurtleState, DrawCommand, TurtlePosition},
    polygon::{generate_default_shapes, TurtlePolygon, TurtleShape},
    speed::TurtleSpeed,
    Request, Response, TurtleShapeName,
};

#[derive(Debug)]
struct TurtleCommand {
    cmd: DrawRequest,
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

    pub(crate) fn do_draw(&mut self, cmd: DrawRequest) {
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

#[derive(Default)]
pub(crate) struct TurtleData {
    queue: VecDeque<TurtleCommand>,       // new elements to draw
    current_command: Option<DrawRequest>, // what we're drawing now
    elements: Vec<DrawCommand>,
    current_shape: CurrentTurtleState,

    current_turtle_id: u64, // which thread to notify on completion
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
    fn convert_command(&mut self, cmd: &DrawRequest) {
        if let Some(command) = self.current_shape.apply(cmd) {
            if matches!(command, DrawCommand::Filler) {
                self.insert_fill = Some(self.elements.len())
            }
            if matches!(command, DrawCommand::DrawPolygon(_)) {
                if let Some(index) = self.insert_fill.take() {
                    self.elements[index] = command;
                }
            } else {
                self.elements.push(command);
            }
        }
    }

    fn draw(&self, context: &Context, gl: &mut GlGraphics) {
        let mut pen_color: TurtleColor = "black".into();
        let mut fill_color: TurtleColor = "black".into();
        let mut pen_width = 0.5;
        let mut iter = self.elements.iter().peekable();

        let mut rotation = self.current_shape.angle();
        let mut pos = self.current_shape.pos();

        while let Some(element) = iter.next() {
            let is_last = iter.peek().is_none() && self.percent < 1.;

            match element {
                DrawCommand::Filler => {}
                DrawCommand::DrawDot(rect, color) => {
                    graphics::ellipse((*color).into(), *rect, context.transform, gl);
                }
                DrawCommand::DrawLine(line) => {
                    let begin = [line.begin[0] as f64, line.begin[1] as f64];
                    let end = if is_last {
                        let endx = begin[0] + (line.end[0] as f64 - begin[0]) * self.percent;
                        let endy = begin[1] + (line.end[1] as f64 - begin[1]) * self.percent;
                        [endx, endy]
                    } else {
                        [line.end[0] as f64, line.end[1] as f64]
                    };
                    if line.pen_down {
                        graphics::line_from_to(
                            pen_color.into(),
                            pen_width,
                            begin,
                            end,
                            context.transform,
                            gl,
                        );
                    }
                    pos = end;
                }
                DrawCommand::DrawPolygon(polygon) => {
                    polygon.draw(&fill_color, context.transform, gl);
                }
                DrawCommand::SetPenColor(pc) => {
                    pen_color = *pc;
                }
                DrawCommand::SetPenWidth(pw) => {
                    pen_width = *pw;
                }
                DrawCommand::SetFillColor(fc) => {
                    fill_color = *fc;
                }
                DrawCommand::SetHeading(start, end) => {
                    if is_last {
                        rotation = *start + (*end - *start) * self.percent;
                    } else {
                        rotation = *end;
                    }
                }
                DrawCommand::Stamp(..) => {}
            }
        }

        let trans = context.transform.trans(pos[0], pos[1]).rot_deg(rotation);

        // draw the turtle
        self.turtle_shape.shape.draw(&fill_color, trans, gl);
    }

    fn is_instantaneous(&self) -> bool {
        if let Some(cmd) = self.current_command.as_ref() {
            matches!(cmd, DrawRequest::InstantaneousDraw(_))
        } else {
            false
        }
    }

    fn update(&mut self, delta_t: f64) {
        let s = self.speed.get();

        self.drawing_done = s == 0
            || match self.progression {
                Progression::Forward => self.percent >= 1.,
                Progression::Reverse => self.percent <= 0.,
            }
            || self.is_instantaneous();

        if self.drawing_done {
            self.percent = 1.;
        } else {
            let multiplier = s as f64;

            match self.progression {
                Progression::Forward => self.percent += delta_t * multiplier,
                Progression::Reverse => self.percent -= delta_t * multiplier,
            }
        }

        if self.drawing_done && self.current_command.is_some() {
            self.drawing_done = false;
            self.fill_poly.update(self.current_shape.pos());
            self.shape_poly.update(self.current_shape.pos());

            if matches!(self.progression, Progression::Reverse) {
                if let Some(element) = self.elements.pop() {
                    match element {
                        DrawCommand::DrawLine(line) => {
                            let start = [line.begin[0] as f64, line.begin[1] as f64];
                            self.current_shape.transform = identity().trans_pos(start);
                        }
                        DrawCommand::SetHeading(start, _) => {
                            self.current_shape.angle = start;
                        }
                        _ => {}
                    }
                }
            }

            let cmd = self.current_command.take().unwrap();

            let _ = self.responder[&self.current_turtle_id].send(if cmd.is_stamp() {
                Response::StampID(self.elements.len() - 1)
            } else {
                Response::Done
            });
        }

        if self.current_command.is_none() && !self.queue.is_empty() {
            let TurtleCommand { cmd, turtle_id } = self.queue.pop_front().unwrap();
            self.current_turtle_id = turtle_id;

            self.convert_command(&cmd);

            if matches!(cmd, DrawRequest::TimedDraw(TimedDrawCmd::Undo)) {
                self.progression = Progression::Reverse;
                self.percent = 1.;
            } else {
                self.progression = Progression::Forward;
                self.percent = 0.;
            }

            self.current_command = Some(cmd);
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
                self.data[which].queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::BackfillPolygon),
                    turtle_id,
                });
            }
            ScreenCmd::EndFill => {
                if !self.data[which].fill_poly.verticies.is_empty() {
                    let polygon = TurtlePolygon::new(&self.data[which].fill_poly.verticies);
                    self.data[which].fill_poly.last_point = None;
                    self.data[which].queue.push_back(TurtleCommand {
                        cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Fill(polygon)),
                        turtle_id,
                    })
                }
            }
            ScreenCmd::Background(TurtleColor::CurrentColor) => {}
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.bgcolor = [r, g, b, 1.];
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data[which].elements.clear();
                self.bgcolor = crate::BLACK;
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data[which].elements.len()
                    && matches!(self.data[which].elements[id], DrawCommand::Stamp(true))
                {
                    self.data[which].elements[id] = DrawCommand::Stamp(false);
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
            ClearDirection::Forward => Either::Right(self.data[which].elements.iter_mut()),
            ClearDirection::Reverse => Either::Left(self.data[which].elements.iter_mut().rev()),
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if cmd.is_stamp() {
                    count -= 1;
                    *cmd = DrawCommand::Stamp(false);
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
            DataCmd::UndoBufferEntries => {
                resp.send(Response::Count(self.data[which].elements.len()))
            }
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
                self.data[which].queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Stamp(true)),
                    turtle_id,
                });
                Ok(())
            }
        };
    }

    fn handle_command(&mut self, which: usize, req: Request) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(which, cmd, req.turtle_id),
            Command::Draw(cmd) => self.data[which].queue.push_back(TurtleCommand {
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
            // std::thread::sleep(std::time::Duration::from_millis(5));

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
