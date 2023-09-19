extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

pub struct Turtle {
    gl: GlGraphics, // OpenGL drawing backend.
    cmds: Vec<Command>,
    which: usize,
    percent: f64,
    is_pen_down: bool,
}

enum Command {
    Forward(f64),
    Right(f64),
    Left(f64),
    PenDown,
    PenUp,
}

impl Turtle {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);

            let mut transform = c.transform.trans(x, y);
            let mut pct = 1.;

            for cmd in 0..(self.which + 1) {
                if cmd == self.which {
                    pct = self.percent;
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

        self.percent += args.dt * 60.;
        if self.percent > 1. {
            self.which += 1;
            self.percent = 0.
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [1000, 1000])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    const _CMDS: [Command; 8] = [
        Command::Forward(100.),
        Command::Right(90.),
        Command::Forward(100.),
        Command::Right(90.),
        Command::Forward(100.),
        Command::Right(90.),
        Command::Forward(100.),
        Command::Left(90.),
    ];

    let mut cmds = vec![
        Command::PenUp,
        Command::Right(180.),
        Command::Forward(729. / 2.),
        Command::Right(180.),
        Command::PenDown,
    ];

    spiky_fractal(&mut cmds, 4, 729.);

    // Create a new game and run it.
    let mut app = Turtle {
        gl: GlGraphics::new(opengl),
        cmds,
        is_pen_down: true,
        which: 0,
        percent: 0.,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}

fn spiky_fractal(cmds: &mut Vec<Command>, order: usize, length: f64) {
    if order == 0 {
        cmds.push(Command::Forward(length));
    } else {
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(60.));
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Right(120.));
        spiky_fractal(cmds, order - 1, length / 3.);
        cmds.push(Command::Left(60.));
        spiky_fractal(cmds, order - 1, length / 3.);
    }
}
