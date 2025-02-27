use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("doing nothing")
        .run(|turtle| {
            turtle.forward(100);
            turtle.undo();
        });
}
