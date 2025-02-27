use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("a basic circle")
        .run(|turtle| {
            turtle.forward(100);
            turtle.right(90);
            turtle.speed(1);
            turtle.pencolor("green");
            turtle.circle(125).with_extent(180);
            turtle.pencolor("grey");
            turtle.circle(50).with_extent(270);
        });
}
