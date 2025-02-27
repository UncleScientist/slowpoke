use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            turtle.teleport(0, -300);
            turtle.speed(1);
            turtle.circle(300).with_steps(3usize);
        });
}
