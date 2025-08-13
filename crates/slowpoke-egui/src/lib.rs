use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    time::Duration,
};

use eframe::CreationContext;
use egui::{vec2, Color32, Painter, Pos2, Rect, Stroke};
use slowpoke::{
    Handler, LineSegment, PopupID, SlowpokeLib, TurtleColor, TurtleDraw, TurtleGui, TurtleID,
    TurtleTask, TurtleUI, TurtleUserInterface,
};

pub type Slowpoke = SlowpokeLib<EguiFramework>;
pub type Turtle = slowpoke::Turtle;
pub use slowpoke::TurtleShapeName; // TODO XXX Fix this -- we shouldn't need to do this?

#[derive(Debug)]
pub struct EguiFramework {
    tt: TurtleTask,
    handler: Handler<EguiUI, EguiInternal>,
}

#[derive(Debug, Default)]
struct EguiInternal;

impl TurtleUI for EguiInternal {
    fn generate_popup(&mut self, _popupdata: &slowpoke::PopupData) -> PopupID {
        todo!()
    }

    fn resize(&mut self, _width: isize, _height: isize) {
        // TODO
    }

    fn set_bg_color(&mut self, _bgcolor: TurtleColor) {
        // TODO
    }
}

#[derive(Default, Debug)]
struct EguiUI;

impl EguiUI {
    fn draw(&self, painter: &Painter, cur_size: &Rect, ops: &[TurtleDraw]) {
        fn points_to_pos(segment: &LineSegment) -> (Pos2, Pos2) {
            (
                Pos2 {
                    x: segment.start.x,
                    y: segment.start.y,
                },
                Pos2 {
                    x: segment.end.x,
                    y: segment.end.y,
                },
            )
        }
        let win_center = vec2(cur_size.max.x / 2.0, cur_size.max.y / 2.0);

        let stroke = Stroke::new(0.25, Color32::WHITE);
        for op in ops {
            match op {
                TurtleDraw::DrawLines(_, _, line_segments) => {
                    let mut line_list = Vec::new();
                    let (start, _) = points_to_pos(&line_segments[0]);
                    line_list.push(start + win_center);
                    for segment in line_segments {
                        let (_, end) = points_to_pos(segment);
                        line_list.push(end + win_center);
                    }
                    painter.line(line_list, stroke);
                }
                TurtleDraw::DrawDot(center, radius, color) => {
                    let center = Pos2 {
                        x: center.x,
                        y: center.y,
                    } + win_center;
                    let color: EguiColor = color.into();
                    painter.circle_filled(center, *radius, color);
                }
                TurtleDraw::DrawText(point2_d, _) => {}
                TurtleDraw::FillPolygon(turtle_color, turtle_color1, _, line_segments) => {}
            }
        }
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
            screen: EguiInternal,
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
                ui.draw(painter, &cur_size, &turtle.ops);
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
                if prog.is_done(pct) {
                    turtle.has_new_cmd = false;
                }
            }
        }

        !done
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct EguiColor(egui::Color32);

impl Deref for EguiColor {
    type Target = egui::Color32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EguiColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<TurtleColor> for EguiColor {
    fn from(value: TurtleColor) -> Self {
        (&value).into()
    }
}

impl From<&TurtleColor> for EguiColor {
    fn from(value: &TurtleColor) -> Self {
        if let TurtleColor::Color(r, g, b) = value {
            EguiColor(egui::Color32::from_rgb(
                (*r * 255.0) as u8,
                (*g * 255.0) as u8,
                (*b * 255.0) as u8,
            ))
        } else {
            todo!()
        }
    }
}

impl From<EguiColor> for TurtleColor {
    fn from(value: EguiColor) -> Self {
        let r = value.0.r() as f32 / 255.0;
        let g = value.0.g() as f32 / 255.0;
        let b = value.0.b() as f32 / 255.0;
        TurtleColor::Color(r, g, b)
    }
}

impl From<&EguiColor> for egui::Color32 {
    fn from(value: &EguiColor) -> Self {
        value.0
    }
}

impl From<EguiColor> for egui::Color32 {
    fn from(value: EguiColor) -> Self {
        value.0
    }
}
