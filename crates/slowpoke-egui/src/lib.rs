use std::{collections::HashMap, time::Duration};

use eframe::CreationContext;
use egui::{pos2, vec2, Color32, Frame, Painter, Pos2, Rect, Stroke};
use slowpoke::{
    DrawCommand, Handler, IndividualTurtle, LineInfo, PopupID, SlowpokeLib, TurtleColor, TurtleGui,
    TurtleID, TurtleTask, TurtleUI, TurtleUserInterface,
};

pub type Slowpoke = SlowpokeLib<EguiFramework>;
pub type Turtle = slowpoke::Turtle;

#[derive(Debug)]
pub struct EguiFramework {
    tt: TurtleTask,
    handler: Handler<EguiUI, EguiInternal>,
}

#[derive(Debug, Default)]
struct EguiInternal;

impl TurtleUI for EguiInternal {
    fn generate_popup(&mut self, popupdata: &slowpoke::PopupData) -> PopupID {
        todo!()
    }

    fn resize(&mut self, width: isize, height: isize) {
        // TODO
    }

    fn set_bg_color(&mut self, bgcolor: TurtleColor) {
        // TODO
    }
}

#[derive(Debug, Default)]
struct EguiUI {
    drawing: Vec<EguiCmd>,
}

#[derive(Debug)]
enum EguiCmd {
    Line(Pos2, Pos2),
}

impl EguiUI {
    fn convert(&mut self, pct: f32, cmds: &[DrawCommand], turtle: &IndividualTurtle<EguiUI>) {
        let mut iter = cmds.iter().peekable();
        let mut tpos = pos2(0.0, 0.0);
        let mut trot = 0f32;

        while let Some(element) = iter.next() {
            let last_element = iter.peek().is_none() && pct < 1.;

            match element {
                DrawCommand::Line(line) => {
                    let (start, end) = Self::start_and_end(last_element, pct, line);
                    tpos = end;
                    self.drawing.push(EguiCmd::Line(start, end));
                }
                _ => {}
            }
        }
    }

    fn draw(&self, painter: &Painter, cur_size: &Rect) {
        let center = vec2(cur_size.max.x / 2.0, cur_size.max.y / 2.0);

        let stroke = Stroke::new(0.25, Color32::WHITE);
        for cmd in &self.drawing {
            match cmd {
                EguiCmd::Line(start, end) => {
                    painter.line(vec![*start + center, *end + center], stroke);
                }
            }
        }
    }

    fn start_and_end(last_element: bool, pct: f32, line: &LineInfo) -> (Pos2, Pos2) {
        (
            pos2(line.begin.x as f32, line.begin.y as f32),
            if last_element {
                let end_x = line.begin.x as f32 + (line.end.x - line.begin.x) as f32 * pct;
                let end_y = line.begin.y as f32 + (line.end.y - line.begin.y) as f32 * pct;
                pos2(end_x, end_y)
            } else {
                pos2(line.end.x as f32, line.end.y as f32)
            },
        )
    }
}

impl TurtleUserInterface for EguiFramework {
    fn start(mut flags: slowpoke::TurtleFlags) {
        let func = flags.start_func.take();
        let title = flags.title.clone();
        let mut tt = TurtleTask::new(&mut flags);
        tt.run_turtle(func.unwrap());

        let mut handler = Handler::<EguiUI, EguiInternal> {
            last_id: TurtleID::default(),
            turtle: HashMap::new(),
            title: format!(" {title} "),
            popups: HashMap::new(),
            screen: EguiInternal {
                ..EguiInternal::default()
            },
        };
        let _ = handler.new_turtle();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([flags.size[0], flags.size[1]]),
            ..Default::default()
        };

        eframe::run_native(
            title.as_str(),
            options,
            Box::new(move |cc| Ok(Box::new(EguiFramework::new(cc, handler, tt)))),
        )
        .expect("failed to start turtle");
    }
}

impl eframe::App for EguiFramework {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tt.tick(&mut self.handler);
        // let frame = Frame {
        // fill: Color32::WHITE,
        // ..Frame::default()
        // };

        self.update_turtles();

        egui::CentralPanel::default().show(ctx, |ui| {
            let cur_size = ctx.screen_rect();
            let painter = ui.painter();

            for turtle in self.handler.turtle.values() {
                let ui = turtle.ui.borrow();
                ui.draw(painter, &cur_size);
            }
        });
        ctx.request_repaint_after(Duration::from_millis(10));
    }
}

impl EguiFramework {
    fn new(_cc: &CreationContext, handler: Handler<EguiUI, EguiInternal>, tt: TurtleTask) -> Self {
        Self { handler, tt }
    }

    fn update_turtles(&mut self) -> bool {
        let mut done = true;

        for (tid, turtle) in &mut self.handler.turtle {
            let (pct, prog) = self.tt.progress(*tid);
            if turtle.has_new_cmd {
                done = false;
                let mut ui = turtle.ui.borrow_mut();
                ui.convert(pct, &turtle.cmds, turtle);
                if prog.is_done(pct) {
                    turtle.has_new_cmd = false;
                }
            }
        }

        !done
    }
}
