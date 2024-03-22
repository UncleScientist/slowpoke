use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("a line")
        .run(|turtle| {
            turtle.pencolor(0.8, 0.3, 0.5);
            // turtle.fillcolor(1., 1., 0.);
            turtle.begin_fill();

            turtle.forward(200);
            turtle.right(120);

            turtle.forward(200);
            turtle.right(120);

            turtle.forward(200);
            turtle.end_fill();
        });
}
