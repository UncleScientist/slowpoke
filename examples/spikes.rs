use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a spiky drawing")
        .run(|turtle| {
            for heading in 0..12 {
                turtle.setheading(heading * 360 / 12);
                turtle.forward(100);
                turtle.goto(0, 0)
            }
        });
}
