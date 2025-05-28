use rand::Rng;
use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(500, 500)
        .with_title("multiple turtles")
        .run(|turtle| {
            turtle.shape("turtle");
            turtle.width(2);
            turtle.speed(3);

            let mut rng = rand::rng();

            loop {
                let dist: f64 = 10. + rng.random::<f64>() * 20.;
                let pos = turtle.pos();
                if pos.x > 200 || pos.x < -200 || pos.y > 200 || pos.y < -200 {
                    let h = turtle.towards(0, 0);
                    turtle.setheading(h);
                } else {
                    let angle: f64 = rng.random::<f64>() * 40. - 20.;
                    turtle.right(angle);
                }
                turtle.forward(dist);
            }
        });
}
