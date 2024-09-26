use rand::*;
use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("multiple turtles")
        .run(|turtle| {
            turtle.shape("turtle");
            turtle.width(2);

            let mut t1 = turtle.hatch();
            t1.shape("turtle");
            t1.right(90);
            t1.pencolor("red");
            t1.fillcolor("red");
            t1.width(2);

            let mut t2 = turtle.hatch();
            t2.shape("turtle");
            t2.right(180);
            t2.pencolor("green");
            t2.fillcolor("green");
            t2.width(2);

            let mut t3 = turtle.hatch();
            t3.shape("turtle");
            t3.left(90);
            t3.pencolor("blue");
            t3.fillcolor("blue");
            t3.width(2);

            let mut tlist = [turtle, &mut t1, &mut t2, &mut t3];

            for t in tlist.iter_mut() {
                t.speed("fastest");
            }

            let mut rng = rand::thread_rng();

            loop {
                for t in tlist.iter_mut() {
                    let dist: f64 = 10. + rng.gen::<f64>() * 20.;
                    let pos = t.pos();
                    if pos.x > 200 || pos.x < -200 || pos.y > 200 || pos.y < -200 {
                        let h = t.towards(0, 0);
                        t.setheading(h);
                    } else {
                        let angle: f64 = rng.gen::<f64>() * 40. - 20.;
                        t.right(angle);
                    }
                    t.forward(dist);
                }
            }
        });
}
