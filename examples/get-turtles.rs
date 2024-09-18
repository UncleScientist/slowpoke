use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("lotsa turtles")
        .run(|turtle| {
            for _ in 0..12 {
                let _ = turtle.hatch();
            }

            for (i, mut t) in turtle.turtles().into_iter().enumerate() {
                t.right(i as u32 * 360 / 12);
                t.forward(i as u32 * 12)
            }

            loop {
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("red");
                    t.forward(0);
                }
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("green");
                    t.forward(0);
                }
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("blue");
                    t.forward(0);
                }
            }
        });
}
