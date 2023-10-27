use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            for _ in 0..4 {
                turtle.forward(100);
                turtle.right(90);
            }
        });
}
