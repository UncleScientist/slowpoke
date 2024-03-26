use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("a parallelogram")
        .run(|turtle| {
            println!("start: {:?}", turtle.pos());

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

            turtle.right(60);
            turtle.forward(150);
            println!("start point again: {:?}", turtle.pos());

            turtle.end_fill();
        });
}
