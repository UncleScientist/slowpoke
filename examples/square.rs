use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("a square")
        .run(|turtle| {
            turtle.speed(2);
            turtle.left(37);
            for _ in 0..4 {
                turtle.forward(100);
                turtle.right(90);
            }
        });
}
