use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("multiple turtles")
        .run(|turtle| {
            let mut t1 = turtle.hatch();
            turtle.forward(100);
            turtle.right(90);
            let mut t2 = turtle.hatch();
            turtle.forward(100);
            turtle.right(90);

            t1.right(45);
            t2.left(45);

            t1.forward(100);
            t2.forward(100);
        });
}
