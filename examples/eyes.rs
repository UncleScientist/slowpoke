use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            fn aankha(turtle: &mut Turtle) {
                turtle.fillcolor("#ffffff");
                turtle.begin_fill();

                turtle.tracer(false);
                let mut a = 2.5;
                for i in 0..120 {
                    if (0..30).contains(&i) || (60..90).contains(&i) {
                        a -= 0.05;
                    } else {
                        a += 0.05;
                    }
                    turtle.left(3);
                    turtle.forward(a);
                }
                turtle.tracer(true);
                turtle.end_fill();
            }

            turtle.bgcolor("#333333");
            turtle.teleport(-100, 100);
            aankha(turtle);
            turtle.teleport(100, 100);
            aankha(turtle);
            turtle.teleport(100, -100);
            aankha(turtle);
            turtle.teleport(-100, -100);
            aankha(turtle);
        });
}
