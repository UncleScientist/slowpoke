// Copied from:
// https://cssmartkids.com/draw-doremon-with-python/
//
// Ported to rust

use slowpoke::{SlowpokeLib, Turtle};

#[allow(clippy::too_many_lines)]
fn main() {
    SlowpokeLib::default()
        .with_size(600, 800)
        .with_title("a doraemon")
        .run(|turtle| {
            fn move_to(turtle: &mut Turtle, x: f32, y: f32) {
                turtle.penup();
                turtle.goto(x, y);
                turtle.pendown();
            }

            fn taauko(turtle: &mut Turtle) {
                turtle.penup();
                turtle.circle(150).with_extent(40);
                turtle.pendown();
                turtle.fillcolor("#00a0de");
                turtle.begin_fill();
                turtle.circle(150).with_extent(280);
                turtle.end_fill();
            }

            fn muflar(turtle: &mut Turtle) {
                turtle.fillcolor("#e70010");
                turtle.begin_fill();
                turtle.setheading(0);
                turtle.forward(200);
                turtle.circle(-5).with_extent(90);
                turtle.forward(10);
                turtle.circle(-5).with_extent(90);
                turtle.forward(207);
                turtle.circle(-5).with_extent(90);
                turtle.forward(10);
                turtle.circle(-5).with_extent(90);

                turtle.end_fill();
            }

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

            fn face(turtle: &mut Turtle) {
                turtle.forward(183);
                turtle.left(45);
                turtle.fillcolor("#ffffff");
                turtle.begin_fill();
                turtle.circle(120).with_extent(100);
                turtle.setheading(180);
                turtle.forward(121);
                turtle.pendown();
                turtle.setheading(215);
                turtle.circle(120).with_extent(100);
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

            fn nak(turtle: &mut Turtle) {
                move_to(turtle, -10., 158.);
                turtle.setheading(315);
                turtle.fillcolor("#e70010");
                turtle.begin_fill();
                turtle.circle(20);
                turtle.end_fill();
            }

            fn mukh(turtle: &mut Turtle) {
                move_to(turtle, 5., 148.);
                turtle.setheading(270);
                turtle.forward(100);
                turtle.setheading(0);
                turtle.circle(120).with_extent(50);
                turtle.setheading(230);
                turtle.circle(-120).with_extent(100);
            }

            fn daari(turtle: &mut Turtle) {
                move_to(turtle, -32., 135.);
                turtle.setheading(165);
                turtle.forward(60);

                move_to(turtle, -32., 125.);
                turtle.setheading(180);
                turtle.forward(60);

                move_to(turtle, -32., 115.);
                turtle.setheading(193);
                turtle.forward(60);

                move_to(turtle, 37., 135.);
                turtle.setheading(15);
                turtle.forward(60);

                move_to(turtle, 37., 125.);
                turtle.setheading(0);
                turtle.forward(60);

                move_to(turtle, 37., 115.);
                turtle.setheading(-13);
                turtle.forward(60);
            }

            fn doraemon(turtle: &mut Turtle) {
                taauko(turtle);
                muflar(turtle);
                face(turtle);
                nak(turtle);
                mukh(turtle);
                daari(turtle);

                move_to(turtle, 0., 0.);
                turtle.setheading(0);
                turtle.penup();
                turtle.circle(150).with_extent(50);
                turtle.pendown();
                turtle.setheading(30);
                turtle.forward(40);
                turtle.setheading(70);
                turtle.circle(-30).with_extent(270);

                turtle.fillcolor("#00a0de");
                turtle.begin_fill();
                turtle.setheading(230);
                turtle.forward(80);
                turtle.setheading(90);
                turtle.circle(1000).with_extent(1);
                turtle.setheading(-89);
                turtle.circle(-1000).with_extent(10);
            }

            turtle.bgcolor("#f0f0f0");
            turtle.width(3);
            turtle.speed(9);
            doraemon(turtle);
            turtle.teleport(100, -300);
            // turtle.write("by CS-SmartKids");
        });
}
