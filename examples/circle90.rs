use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("a basic circle")
        .run(|turtle| {
            turtle.circle(125).with_extent(90);
        });
}
