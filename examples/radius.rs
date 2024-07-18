use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(600, 400)
        .with_title("a corner radius")
        .run(|turtle| {
            turtle.speed(1);
            turtle.fillcolor("#e70010");
            turtle.begin_fill();
            turtle.setheading(0);
            turtle.forward(200);
            turtle.extent(90).circle(-5);
            turtle.forward(10);
            turtle.extent(90).circle(-5);
            turtle.forward(207);
            turtle.extent(90).circle(-5);
            turtle.forward(10);
            turtle.extent(90).circle(-5);
            /*
            turtle.forward(100);
            turtle.circle(-30, 90, 20);
            turtle.forward(100);
                */
        });
}
