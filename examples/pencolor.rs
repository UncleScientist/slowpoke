use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("pen colors")
        .run(|turtle| {
            let mut red = 0.1;
            turtle.speed(1);
            while red < 0.8 {
                red += 0.02;
                turtle.pencolor((red, 0., 0.));
                turtle.forward(100);
                turtle.right(180 - 17);
            }
        });
}
