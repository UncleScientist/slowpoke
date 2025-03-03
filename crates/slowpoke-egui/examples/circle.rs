#![allow(clippy::cast_precision_loss)]

use slowpoke_egui::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            turtle.pencolor("black");
            turtle.bgcolor("grey");
            turtle.speed(1);
            turtle.fillcolor("light green");
            for i in 3i16..20 {
                turtle.circle(10 + i * 10).with_steps(i.unsigned_abs());
            }
            for i in 3i16..20 {
                turtle.circle(-(10 + i * 10)).with_steps(i.unsigned_abs());
            }
        });
}
