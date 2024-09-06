use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("some text")
        .run(|turtle| {
            turtle.write("hello");
            turtle.forward(100);
            turtle.write("world");
        });
}
