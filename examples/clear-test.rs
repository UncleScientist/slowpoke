use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.penwidth(4);
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
