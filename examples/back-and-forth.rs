use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.forward(100);
            turtle.goto(0, 0);

            turtle.penup();
            turtle.goto(100, 0);
            turtle.setheading(180);
            turtle.dot();
        });
}
