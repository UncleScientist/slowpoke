use rand::Rng;
use slowpoke::{Shape, Slowpoke};

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("polygon test")
        .run(|turtle| {
            println!("before: {:?}", turtle.getshapes());
            let shape =
                Shape::polygon(&[[100., 100.], [-100., 100.], [-100., -100.], [100., -100.]]);
            turtle.register_shape("different square", shape);
            println!(" after: {:?}", turtle.getshapes());
            turtle.shape("different square");
            std::thread::sleep(std::time::Duration::from_millis(500));

            turtle.shape("classic");

            let mut rng = rand::thread_rng();

            turtle.tracer(false);
            turtle.penup();
            turtle.begin_poly();
            for _ in 0..5 {
                if rng.gen::<f64>() < 0.5 {
                    turtle.right(90. + rng.gen::<f64>() * 45.);
                } else {
                    turtle.left(90. + rng.gen::<f64>() * 45.);
                }
                turtle.forward(30. + rng.gen::<f64>() * 30.);
            }
            turtle.end_poly();
            turtle.goto(0, 0);
            turtle.pendown();
            turtle.tracer(true);

            let poly = turtle.get_poly();
            println!("poly: {poly:?}");
            turtle.register_shape("oddball", poly);

            turtle.shape("oddball");
            for _ in 0..4 {
                turtle.forward(100);
                turtle.right(90);
            }

            let mut compound = Shape::compound();
            compound.addcomponent(
                &[
                    [50., -20.],
                    [30., 20.],
                    [-50., 20.],
                    [-30., -20.],
                    [50., -20.],
                ],
                "red",
                "blue",
            );
            turtle.register_shape("pg", compound);
            turtle.shape("pg");
            turtle.bye();
        });
}
