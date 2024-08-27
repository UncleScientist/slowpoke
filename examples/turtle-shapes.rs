use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a square stamp")
        .run(|turtle| {
            turtle.penup();
            turtle.goto(-100, 0);
            for shape in ["classic", "arrow", "circle", "square", "triangle"] {
                turtle.shape(shape);
                turtle.stamp();
                turtle.forward(50);
            }
            turtle.hideturtle();
        });
}
