use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("clear test")
        .run(|turtle| {
            turtle.width(4);
            turtle.fillcolor("red");
            turtle.pencolor("blue");
            turtle.forward(100);
            turtle.right(55);
            turtle.forward(100);
            turtle.right(55);
            turtle.clear();
            turtle.forward(100);
        });
}
