// Copied from:
// https://cssmartkids.com/draw-doremon-with-python/
//
// Ported to rust

use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(600, 800)
        .with_title("a doraemon")
        .run(|turtle| {
            fn taauko(turtle: &mut Turtle) {
                turtle.penup();
                turtle.circle(150, 40, 30);
                turtle.pendown();
                turtle.fillcolor("#00a0de");
                turtle.begin_fill();
                turtle.circle(150, 280, 30);
                turtle.end_fill();
            }

            fn muflar(turtle: &mut Turtle) {
                turtle.fillcolor("#e70010");
                turtle.begin_fill();
                turtle.setheading(0);
                turtle.forward(200);
                turtle.circle(-5, 90, 10);
                turtle.forward(10);
                turtle.circle(-5, 90, 10);
                turtle.forward(207);
                turtle.circle(-5, 90, 10);
                turtle.forward(10);
                turtle.circle(-5, 90, 10);

                turtle.forward(7); // FIXME
                turtle.forward(-7); // FIXME

                turtle.end_fill();
            }

            fn aankha(turtle: &mut Turtle) {
                turtle.fillcolor("#ffffff");
                turtle.begin_fill();

                //TODO: turtle.tracer(false);
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
                //TODO: turtle.tracer(true);
                turtle.end_fill();
            }

            fn face(turtle: &mut Turtle) {
                turtle.forward(183);
                turtle.left(45);
                turtle.fillcolor("#ffffff");
                turtle.begin_fill();
                turtle.circle(120, 100, 25);
                turtle.setheading(180);
                turtle.forward(121);
                turtle.pendown();
                turtle.setheading(215);
                turtle.circle(120, 100, 25);
                turtle.end_fill();
                turtle.teleport(63.56, 218.24);
                turtle.setheading(90);
                aankha(turtle);
                turtle.setheading(180);
                turtle.penup();
                turtle.forward(60);
                turtle.pendown();
                turtle.setheading(90);
                aankha(turtle);
                turtle.penup();
                turtle.setheading(180);
                turtle.forward(64);
            }

            fn doraemon(turtle: &mut Turtle) {
                taauko(turtle);
                muflar(turtle);
                face(turtle);
            }

            turtle.bgcolor("#f0f0f0");
            turtle.penwidth(3);
            turtle.speed(9);
            doraemon(turtle);
            turtle.teleport(100, -300);
            // turtle.write("by CS-SmartKids");
        });
}
