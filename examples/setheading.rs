use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("heading demo")
        .run(|turtle| {
            turtle.speed(1);
            // draw to the right
            turtle.setheading(0);
            turtle.forward(100);
            turtle.teleport(0, 0);

            // draw "south" or down
            turtle.setheading(270);
            turtle.forward(100);
            turtle.teleport(0, 0);

            // draw "north" or up
            turtle.setheading(90);
            turtle.forward(100);
            turtle.teleport(0, 0);

            // draw "west" or to the left
            turtle.setheading(180);
            turtle.forward(100);
            turtle.teleport(0, 0);

            turtle.left(30);
        });
}
