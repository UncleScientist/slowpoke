use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    EventSettings, Events, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, WindowSettings,
};

pub struct Turtle {
    gl: GlGraphics, // OpenGL drawing backend.
    window: GlutinWindow,
    cmds: Vec<Command>,
    which: usize,
    percent: f64,
    is_pen_down: bool,
}

impl Default for Turtle {
    fn default() -> Self {
        Self::new()
    }
}

impl Turtle {
    pub fn new() -> Self {
        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create a Glutin window.
        let window: GlutinWindow = WindowSettings::new("spinning-square", [1000, 1000])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        Self {
            gl: GlGraphics::new(opengl),
            window,
            cmds: Vec::new(),
            is_pen_down: true,
            which: 0,
            percent: 0.,
        }
    }

    pub fn insert_commands(&mut self, cmds: Vec<Command>) {
        self.cmds = cmds;
    }

    pub fn run(&mut self) {
        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut self.window) {
            if let Some(args) = e.render_args() {
                self.render(&args);
            }

            if let Some(args) = e.update_args() {
                self.update(&args);
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);

            let mut transform = c.transform.trans(x, y).rot_deg(-90.);
            let mut pct = 1.;
            let mut full = true;
            let mut deg: f64 = -90.;

            for cmd in 0..(self.which + 1) {
                if cmd == self.which {
                    pct = self.percent;
                    full = pct >= 1.;
                }

                if full {
                    deg += self.cmds[cmd].get_rotation();
                }

                if cmd >= self.cmds.len() {
                    break;
                }

                match &self.cmds[cmd] {
                    Command::Forward(dist) => {
                        if self.is_pen_down {
                            line_from_to(BLACK, 1.0, [0., 0.], [*dist * pct, 0.], transform, gl);
                        }
                        transform = transform.trans(*dist * pct, 0.);
                    }
                    Command::Right(deg) => transform = transform.rot_deg(*deg * pct),
                    Command::Left(deg) => transform = transform.rot_deg(-deg * pct),
                    Command::PenDown => self.is_pen_down = true,
                    Command::PenUp => self.is_pen_down = false,
                    Command::GoTo(xpos, ypos) => {
                        self.percent = 1.;
                        transform = c.transform.trans(*xpos + x, *ypos + y).rot_deg(deg);
                    }
                }
            }

            let square = rectangle::square(0.0, 0.0, 10.0);
            ellipse(BLACK, square, transform.trans(-5., -5.), gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.cmds.is_empty() {
            return;
        }

        if self.which == self.cmds.len() - 1 && self.percent >= 1. {
            return;
        }

        self.percent += args.dt * 30.;
        if self.percent >= 1. {
            self.which += 1;
            self.percent = 0.
        }
    }
}

pub enum Command {
    Forward(f64),
    Right(f64),
    Left(f64),
    PenDown,
    PenUp,
    GoTo(f64, f64),
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
