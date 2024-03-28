use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            const COUNT: usize = 10;
            for _ in 0..COUNT {
                turtle.forward(50);
                turtle.left(87);
            }
            for _ in 0..COUNT * 2 {
                turtle.undo();
            }
        });
}
