use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("simple undo")
        .run(|turtle| {
            turtle.right(23);
            turtle.forward(100);
            turtle.left(47);
            turtle.backward(100);

            turtle.undo();
            turtle.undo();
            turtle.undo();
            turtle.undo();
        });
}
