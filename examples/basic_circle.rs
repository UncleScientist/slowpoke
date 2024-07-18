use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a basic circle")
        .run(|turtle| {
            turtle.forward(100);
            turtle.right(90);
            turtle.speed(1);
            turtle.pencolor("green");
            turtle.extent(180).circle(125);
            turtle.pencolor("grey");
            turtle.extent(270).circle(50);
        });
}
