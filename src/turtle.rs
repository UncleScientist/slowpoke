use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
};

use iced::keyboard::{Event::KeyPressed, Key};
use iced::window::Event::Resized;
use iced::{
    event, executor, mouse,
    widget::{
        canvas::{self, fill::Rule, stroke, Cache, Fill, Path, Stroke},
        Canvas,
    },
    window, Application, Color, Event, Length, Point, Rectangle, Renderer, Settings, Size,
    Subscription, Theme,
};
use lyon_tessellation::geom::{euclid::default::Transform2D, Angle};

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

type IcedCommand<T> = iced::Command<T>;

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
    pub tracer: bool,
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

        let _ = TurtleTask::run(Settings {
            antialiasing: true,
            flags,
            window: window::Settings {
                size: Size::new(xsize, ysize),
                ..Default::default()
            },
            ..Settings::default()
        });
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
            tracer: true,
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
        let is_data_cmd = matches!(cmd, Command::Data(_));
        if let Command::Draw(DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t))) = &cmd
        {
            self.tracer = *t;
        }

        if self.issue_command.send(self.req(cmd)).is_ok() {
            if self.tracer {
                if let Ok(result) = self.command_complete.recv() {
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
                        Ok(response) => assert!(matches!(response, Response::Done)),
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
enum Progression {
    #[default]
    Forward,
    Reverse,
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
pub(crate) struct TurtleData {
    queue: VecDeque<TurtleCommand>,       // new elements to draw
    current_command: Option<DrawRequest>, // what we're drawing now
    elements: Vec<DrawCommand>,
    current_shape: CurrentTurtleState,

    current_turtle_id: u64, // which thread to notify on completion
    percent: f32,
    progression: Progression,
    insert_fill: Option<usize>,
    responder: HashMap<u64, Sender<Response>>,
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

impl TurtleData {
    fn new() -> Self {
        Self {
            percent: 2.,
            tracer: true,
            ..Self::default()
        }
    }

    fn convert_command(&mut self, cmd: &DrawRequest) {
        if let Some(command) = self.current_shape.apply(cmd) {
            if matches!(command, DrawCommand::Filler) {
                self.insert_fill = Some(self.elements.len())
            }
            match &command {
                DrawCommand::Line(lineinfo) => {
                    self.fill_poly.update(lineinfo.end);
                    self.shape_poly.update(lineinfo.end);
                    self.elements.push(command);
                }
                DrawCommand::Circle(circle) => {
                    for c in circle {
                        self.fill_poly.update([c.x, c.y].into());
                        self.shape_poly.update([c.x, c.y].into());
                    }
                    self.elements.push(command);
                }
                DrawCommand::DrawPolygon(_) => {
                    if let Some(index) = self.insert_fill.take() {
                        self.elements[index] = command;
                        self.elements.push(DrawCommand::EndFill(index));
                    }
                }
                DrawCommand::StampTurtle => {
                    self.elements.push(DrawCommand::DrawPolyAt(
                        self.turtle_shape.shape.clone(),
                        self.current_shape.pos(),
                        self.current_shape.angle,
                    ));
                }
                _ => {
                    self.elements.push(command);
                }
            }
        }
    }

    fn is_instantaneous(&self) -> bool {
        if let Some(cmd) = self.current_command.as_ref() {
            matches!(cmd, DrawRequest::InstantaneousDraw(_))
        } else {
            false
        }
    }

    fn time_passes(&mut self, delta_t: f32) {
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
            let multiplier = s as f32;

            match self.progression {
                Progression::Forward => self.percent += delta_t * multiplier,
                Progression::Reverse => self.percent -= delta_t * multiplier,
            }
        }

        if !self.tracer && !self.queue.is_empty() {
            while !self.tracer && !self.queue.is_empty() {
                self.drawing_done = true;
                self.do_next_command();
            }
        }
        self.do_next_command();
    }

    fn do_next_command(&mut self) {
        if self.drawing_done && self.current_command.is_some() {
            self.drawing_done = false;

            if matches!(self.progression, Progression::Reverse) {
                if let Some(element) = self.elements.pop() {
                    match element {
                        DrawCommand::EndFill(pos) => {
                            self.elements[pos] = DrawCommand::Filler;
                        }
                        DrawCommand::Line(line) => {
                            let x = line.begin.x as f32;
                            let y = line.begin.y as f32;
                            self.current_shape.transform = Transform2D::translation(x, y);
                        }
                        DrawCommand::SetHeading(start, _) => {
                            self.current_shape.angle = start;
                        }
                        _ => {}
                    }
                }
            }

            let cmd = self.current_command.take().unwrap();

            if cmd.tracer_true() {
                self.respond_immediately = false;
            }

            if !self.respond_immediately {
                self.send_response(self.current_turtle_id, cmd.is_stamp());
            }

            if cmd.tracer_false() {
                self.respond_immediately = true;
            }
        }

        if self.current_command.is_none() && !self.queue.is_empty() {
            let TurtleCommand { cmd, turtle_id } = self.queue.pop_front().unwrap();
            self.current_turtle_id = turtle_id;

            self.convert_command(&cmd);

            if let DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Tracer(t)) = &cmd {
                self.tracer = *t;
            }

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

    fn send_response(&mut self, turtle_id: u64, is_stamp: bool) {
        let _ = self.responder[&turtle_id].send(if is_stamp {
            Response::StampID(self.elements.len())
        } else {
            Response::Done
        });
    }

    fn draw_iced(&self, frame: &mut canvas::Frame) {
        let mut pencolor = Color::BLACK;
        let mut penwidth = 1.0;
        let mut fillcolor = Color::BLACK;
        let pct = self.percent;

        let mut tpos = [0f32, 0f32];
        let mut trot = 0f32;

        let mut iter = self.elements.iter().peekable();

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && self.percent < 1.;
            match element {
                DrawCommand::Filler => {}
                DrawCommand::StampTurtle => todo!(),
                DrawCommand::Line(l) => {
                    let start: Point = [l.begin.x as f32, l.begin.y as f32].into();
                    let end: Point = if last_element {
                        let endx = l.begin.x as f32 + (l.end.x - l.begin.x) as f32 * pct;
                        let endy = l.begin.y as f32 + (l.end.y - l.begin.y) as f32 * pct;
                        tpos = [endx, endy];
                        [endx, endy]
                    } else {
                        tpos = [l.end.x as f32, l.end.y as f32];
                        [l.end.x as f32, l.end.y as f32]
                    }
                    .into();
                    if l.pen_down {
                        let path = Path::new(|b| {
                            b.move_to(start);
                            b.line_to(end);
                        });
                        frame.stroke(
                            &path,
                            Stroke {
                                style: stroke::Style::Solid(pencolor),
                                width: penwidth,
                                ..Stroke::default()
                            },
                        );
                    }
                }
                DrawCommand::SetPenColor(pc) => {
                    pencolor = pc.into();
                }
                DrawCommand::SetPenWidth(pw) => penwidth = *pw,
                DrawCommand::SetFillColor(fc) => {
                    fillcolor = fc.into();
                }
                DrawCommand::DrawPolygon(p) => {
                    let path = p.get_path();
                    frame.fill(
                        path,
                        Fill {
                            style: stroke::Style::Solid(fillcolor),
                            rule: Rule::EvenOdd,
                        },
                    );
                }
                DrawCommand::SetHeading(start, end) => {
                    let rotation = if last_element {
                        *start + (*end - *start) * self.percent
                    } else {
                        *end
                    };
                    trot = rotation;
                }
                DrawCommand::DrawDot(center, radius, color) => {
                    let center: Point = Point::new(center.x, center.y);
                    let circle = Path::circle(center, *radius);
                    frame.fill(
                        &circle,
                        Fill {
                            style: stroke::Style::Solid(color.into()),
                            rule: Rule::NonZero,
                        },
                    );
                }
                DrawCommand::EndFill(_) => {}
                DrawCommand::DrawPolyAt(polygon, pos, angle) => {
                    let path = polygon.get_path();
                    let angle = Angle::degrees(*angle);
                    let xform = Transform2D::rotation(angle).then_translate([pos.x, pos.y].into());
                    let path = path.transform(&xform);
                    frame.fill(
                        &path,
                        Fill {
                            style: stroke::Style::Solid(fillcolor),
                            rule: Rule::NonZero,
                        },
                    );
                    frame.stroke(
                        &path,
                        Stroke {
                            style: stroke::Style::Solid(pencolor),
                            width: penwidth,
                            ..Stroke::default()
                        },
                    );
                }
                DrawCommand::Circle(points) => {
                    if points[0].pen_down {
                        let (total, subpercent) = if last_element {
                            let partial = (points.len() - 1) as f32 * self.percent;
                            (partial.floor() as usize, (partial - partial.floor()))
                        } else {
                            (points.len() - 1, 1_f32)
                        };
                        let path = Path::new(|b| {
                            let (_, start) = points[0].get_data();

                            b.move_to(start.into());

                            let mut iter = points.windows(2).take(total + 1).peekable();
                            while let Some(p) = iter.next() {
                                let (end_angle, end) = p[1].get_data();
                                let last_segment = iter.peek().is_none();
                                let end = if last_element && last_segment {
                                    let (_, begin) = p[0].get_data();
                                    let endx = begin[0] + (end[0] - begin[0]) * subpercent;
                                    let endy = begin[1] + (end[1] - begin[1]) * subpercent;
                                    tpos = [endx, endy];
                                    [endx, endy]
                                } else {
                                    end
                                };
                                b.line_to(end.into());
                                trot = end_angle;
                            }
                        });

                        frame.stroke(
                            &path,
                            Stroke {
                                style: stroke::Style::Solid(pencolor),
                                width: penwidth,
                                ..Stroke::default()
                            },
                        );
                    }
                }
            }
        }

        if !self.turtle_invisible {
            let path = self.turtle_shape.shape.get_path();
            let angle = Angle::degrees(trot);
            let transform = Transform2D::rotation(angle).then_translate(tpos.into());
            let path = path.transform(&transform);
            frame.fill(
                &path,
                Fill {
                    style: stroke::Style::Solid(fillcolor),
                    rule: Rule::EvenOdd,
                },
            );
            frame.stroke(
                &path,
                Stroke {
                    style: stroke::Style::Solid(pencolor),
                    width: penwidth,
                    ..Stroke::default()
                },
            );
        }
    }
}

struct TurtleTask {
    cache: Cache,
    flags: TurtleFlags,
    turtle_num: u64,
    bgcolor: TurtleColor,
    data: Vec<TurtleData>,
    shapes: HashMap<String, TurtleShape>,
    winsize: Size,
    wcmds: Vec<IcedCommand<Message>>,
}

#[derive(Debug)]
enum Message {
    Tick,
    Event(Event),
}

type TurtleStartFunc = dyn FnOnce(&mut Turtle) + Send + 'static;

#[derive(Default)]
struct TurtleFlags {
    start_func: Option<Box<TurtleStartFunc>>,
    issue_command: Option<Sender<Request>>,
    receive_command: Option<Receiver<Request>>,
    title: String,
    size: [f32; 2],
}

impl Application for TurtleTask {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = TurtleFlags;

    fn new(mut flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let func = flags.start_func.take();
        let winsize = flags.size.into();

        let mut tt = TurtleTask {
            cache: Cache::default(),
            flags,
            turtle_num: 0,
            bgcolor: TurtleColor::from("white"),
            shapes: generate_default_shapes(),
            winsize,
            data: vec![TurtleData::new()],
            wcmds: Vec::new(),
        };
        tt.run_turtle(func.unwrap());
        (tt, IcedCommand::none())
    }

    fn title(&self) -> String {
        self.flags.title.clone()
    }

    fn update(&mut self, message: Self::Message) -> IcedCommand<Self::Message> {
        match message {
            Message::Tick => self.tick(),
            Message::Event(event) => self.handle_event(event),
        }
        IcedCommand::batch(self.wcmds.drain(..).collect::<Vec<_>>())
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick),
            event::listen().map(Message::Event),
        ])
    }
}

impl<Message> canvas::Program<Message> for TurtleTask {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<<Renderer as canvas::Renderer>::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            frame.fill_rectangle(
                [0., 0.].into(),
                bounds.size(),
                Fill {
                    style: stroke::Style::Solid((&self.bgcolor).into()),
                    rule: Rule::NonZero,
                },
            );
            frame.translate([center.x, center.y].into());
            for turtle in &self.data {
                turtle.draw_iced(frame);
            }
        });
        vec![geometry]
    }
}

impl TurtleTask {
    fn handle_event(&mut self, event: Event) {
        let mut work = Vec::new();

        match event {
            Event::Window(window::Id::MAIN, Resized { width, height }) => {
                self.winsize = Size::new(width as f32, height as f32);
            }
            Event::Keyboard(KeyPressed { key, .. }) => {
                if let Key::Character(s) = key.as_ref() {
                    let ch = s.chars().next().unwrap();
                    for (idx, turtle) in self.data.iter().enumerate() {
                        if let Some(func) = turtle.onkeypress.get(&ch).copied() {
                            work.push((idx, func, ch));
                        }
                    }
                }
            }
            _ => {}
        }

        for (idx, func, key) in work {
            let mut new_turtle = self.spawn_turtle(idx);
            let _ = std::thread::spawn(move || func(&mut new_turtle, key));
        }
    }

    fn run_turtle<F: FnOnce(&mut Turtle) + Send + 'static>(&mut self, func: F) {
        let mut turtle = self.spawn_turtle(0);
        let _ = std::thread::spawn(move || func(&mut turtle));
    }

    fn tick(&mut self) {
        self.cache.clear();

        while let Ok(req) = self.flags.receive_command.as_ref().unwrap().try_recv() {
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

        for turtle in self.data.iter_mut() {
            turtle.time_passes(0.01); // TODO: use actual time delta
        }
    }

    pub(crate) fn hatch_turtle(&mut self) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.turtle_num += 1;
        let newid = self.turtle_num;

        let mut td = TurtleData::new();
        td.responder.insert(newid, finished);
        self.data.push(td);

        Turtle::init(
            self.flags.issue_command.as_ref().unwrap().clone(),
            command_complete,
            newid,
        )
    }

    fn spawn_turtle(&mut self, which: usize) -> Turtle {
        let (finished, command_complete) = mpsc::channel();
        self.turtle_num += 1;
        let newid = self.turtle_num;
        self.data[which].responder.insert(newid, finished);

        Turtle::init(
            self.flags.issue_command.as_ref().unwrap().clone(),
            command_complete,
            newid,
        )
    }

    fn screen_cmd(&mut self, which: usize, cmd: ScreenCmd, turtle_id: u64) {
        let resp = self.data[which].responder.get(&turtle_id).unwrap().clone();
        match cmd {
            ScreenCmd::SetSize(s) => {
                self.wcmds
                    .push(window::resize::<Message>(window::Id::MAIN, s));
                self.winsize = s; // TODO: wait until resize is complete before saving
                let _ = resp.send(Response::Done); // TODO: don't respond until resize event
            }
            ScreenCmd::ShowTurtle(t) => {
                self.data[which].turtle_invisible = !t;
                let _ = resp.send(Response::Done);
            }
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
            ScreenCmd::Background(TurtleColor::CurrentColor) => {
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::Background(TurtleColor::Color(r, g, b)) => {
                self.bgcolor = [r, g, b, 1.].into();
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearScreen => {
                self.data[which].elements.clear();
                self.bgcolor = TurtleColor::from("black");
                let _ = resp.send(Response::Done);
            }
            ScreenCmd::ClearStamp(id) => {
                if id < self.data[which].elements.len()
                    && matches!(self.data[which].elements[id], DrawCommand::DrawPolyAt(..))
                {
                    self.data[which].elements[id] = DrawCommand::Filler;
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
                    *cmd = DrawCommand::Filler
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
            DataCmd::GetScreenSize => resp.send(Response::ScreenSize(self.winsize)),
            DataCmd::Visibility => {
                resp.send(Response::Visibility(!self.data[which].turtle_invisible))
            }
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
                let curpos: ScreenPosition<f32> = self.data[which].current_shape.pos();
                let x = xpos - curpos.x;
                let y = ypos + curpos.y;
                let heading = y.atan2(x) * 360. / (2.0 * PI);

                resp.send(Response::Heading(heading))
            }
            DataCmd::Position => {
                resp.send(Response::Position(self.data[which].current_shape.pos()))
            }
            DataCmd::Heading => {
                resp.send(Response::Heading(self.data[which].current_shape.angle()))
            }
            DataCmd::Stamp => {
                self.data[which].queue.push_back(TurtleCommand {
                    cmd: DrawRequest::InstantaneousDraw(InstantaneousDrawCmd::Stamp),
                    turtle_id,
                });
                Ok(())
            }
        };
    }

    fn draw_cmd(&mut self, which: usize, cmd: DrawRequest, turtle_id: u64) {
        let is_stamp = cmd.is_stamp();
        self.data[which]
            .queue
            .push_back(TurtleCommand { cmd, turtle_id });

        // FIXME: data commands (Command::Data(_)) require all queued entries to be
        // processed before sending a response, even if `respond_immediately` is set
        if self.data[which].respond_immediately {
            self.data[which].send_response(turtle_id, is_stamp);
        }
    }

    fn handle_command(&mut self, which: usize, req: Request) {
        match req.cmd {
            Command::Screen(cmd) => self.screen_cmd(which, cmd, req.turtle_id),
            Command::Draw(cmd) => self.draw_cmd(which, cmd, req.turtle_id),
            Command::Input(cmd) => self.input_cmd(which, cmd, req.turtle_id),
            Command::Data(cmd) => self.data_cmd(which, cmd, req.turtle_id),
            Command::Hatch => {
                let new_turtle = self.hatch_turtle();
                let resp = &self.data[which].responder[&req.turtle_id];
                let _ = resp.send(Response::Turtle(new_turtle));
            }
        }
    }
}
