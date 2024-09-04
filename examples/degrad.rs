use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("degrees and radians")
        .run(|turtle| {
            turtle.left(90);
            println!("before: {}", turtle.heading());
            turtle.radians();
            println!(" after: {}", turtle.heading());
        });
}
