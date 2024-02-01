use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a square")
        .run(|turtle| {
            for _ in 0..4 {
                turtle.forward(100);
                turtle.right(90);
            }
        });
}
