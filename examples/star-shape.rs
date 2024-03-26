use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.pencolor((1., 0., 0.));
            turtle.fillcolor((1., 1., 0.));
            turtle.begin_fill();
            loop {
                turtle.forward(200);
                turtle.left(170);
                let pos = turtle.pos();
                let dist = ((pos[0] * pos[0] + pos[1] * pos[1]) as f64).sqrt();
                if dist < 1.0 {
                    break;
                }
            }

            turtle.end_fill();
        });
}
