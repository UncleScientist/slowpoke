use std::time::Duration;

use rand::*;
use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("multiple turtles")
        .run(|turtle| {
            turtle.penwidth(2);

            let mut t1 = turtle.hatch();
            t1.right(90);
            t1.pencolor("red");
            t1.fillcolor("red");
            t1.penwidth(2);

            let mut t2 = turtle.hatch();
            t2.right(180);
            t2.pencolor("green");
            t2.fillcolor("green");
            t2.penwidth(2);

            let mut t3 = turtle.hatch();
            t3.left(90);
            t3.pencolor("blue");
            t3.fillcolor("blue");
            t3.penwidth(2);

            let mut tlist = [turtle, &mut t1, &mut t2, &mut t3];

            for t in tlist.iter_mut() {
                t.speed("fastest");
            }

            for t in tlist.iter_mut() {
                t.ontimer(turtle_thread, 250);
            }
        });
}

fn turtle_thread(turtle: &mut Turtle, _duration: Duration) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() > 0.95 {
        turtle.clear();
    } else {
        let dist: f64 = 10. + rng.gen::<f64>() * 20.;
        let pos = turtle.pos();
        if pos.x > 200 || pos.x < -200 || pos.y > 200 || pos.y < -200 {
            let h = turtle.towards(0, 0);
            turtle.setheading(h);
        } else {
            let angle: f64 = rng.gen::<f64>() * 40. - 20.;
            turtle.right(angle);
        }
        turtle.forward(dist);
    }

    turtle.ontimer(turtle_thread, 150);
}
