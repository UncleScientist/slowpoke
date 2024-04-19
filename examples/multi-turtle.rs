use rand::*;
use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("multiple turtles")
        .run(|turtle| {
            let mut t1 = turtle.hatch();
            t1.right(90);
            let mut t2 = turtle.hatch();
            t2.right(180);
            let mut t3 = turtle.hatch();
            t3.left(90);
            let mut tlist = vec![turtle, &mut t1, &mut t2, &mut t3];

            for t in tlist.iter_mut() {
                t.speed("fastest");
            }

            let mut rng = rand::thread_rng();

            loop {
                for t in tlist.iter_mut() {
                    let dist: f64 = 10. + rng.gen::<f64>() * 20.;
                    t.forward(dist);
                    let pos = t.pos();
                    if pos[0] > 200 || pos[0] < -200 || pos[1] > 200 || pos[1] < -200 {
                        let h = t.towards(0, 0);
                        t.setheading(h);
                    } else {
                        let angle: f64 = rng.gen::<f64>() * 40. - 20.;
                        t.right(angle);
                    }
                }
            }
        });
}
