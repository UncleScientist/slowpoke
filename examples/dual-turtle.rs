use rand::Rng;
use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
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

            let mut tlist = [turtle, &mut t1];

            for t in &mut tlist {
                t.speed(1);
            }

            let mut rng = rand::rng();

            loop {
                for t in &mut tlist {
                    let dist: f64 = 10. + rng.random::<f64>() * 20.;
                    let pos = t.pos();
                    if pos.x > 200 || pos.x < -200 || pos.y > 200 || pos.y < -200 {
                        let h = t.towards(0, 0);
                        t.setheading(h);
                    } else {
                        let angle: f64 = rng.random::<f64>() * 40. - 20.;
                        t.right(angle);
                    }
                    t.forward(dist);
                }
            }
        });
}
