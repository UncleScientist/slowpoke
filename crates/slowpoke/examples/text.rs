use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("some text")
        .run(|turtle| {
            for _ in 0..3 {
                turtle.right(90);
                turtle.write("hello");
                turtle.forward(100);
                turtle.write("world");
            }
        });
}
