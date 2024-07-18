use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            turtle.teleport(0, -300);
            turtle.speed(1);
            turtle.steps(3).circle(300);
        });
}
