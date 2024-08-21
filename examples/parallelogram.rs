use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("a parallelogram")
        .run(|turtle| {
            println!("start: {:?}", turtle.pos());
            turtle.speed(6);

            turtle.begin_poly();

            turtle.pencolor((0.4, 0.8, 0.5));
            turtle.fillcolor((1., 1., 0.));
            turtle.begin_fill();

            turtle.right(30);
            turtle.forward(100);
            println!("second point: {:?}", turtle.pos());

            turtle.right(60);
            turtle.forward(150);
            println!("third point: {:?}", turtle.pos());

            turtle.right(120);
            turtle.forward(100);
            println!("fourth point: {:?}", turtle.pos());
            println!(
                "if we wanted to go to 0, 0, heading would be {}",
                turtle.towards(0, 0)
            );

            turtle.end_poly();

            turtle.right(60);
            turtle.forward(150);
            println!("start point again: {:?}", turtle.pos());

            turtle.end_fill();

            turtle.backward(150);

            println!("Polygon is {:?}", turtle.get_poly());
        });
}
