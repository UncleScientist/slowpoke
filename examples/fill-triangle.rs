use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("a line")
        .run(|turtle| {
            println!("start: {:?}", turtle.pos());

            turtle.pencolor(0.8, 0.3, 0.5);
            // turtle.fillcolor(1., 1., 0.);
            turtle.begin_fill();

            turtle.forward(200);
            println!("second point: {:?}", turtle.pos());

            std::thread::sleep(std::time::Duration::from_millis(500));

            turtle.right(120);
            turtle.forward(200);
            println!("third point: {:?}", turtle.pos());

            std::thread::sleep(std::time::Duration::from_millis(500));

            turtle.right(120);
            turtle.forward(200);
            println!("back to the start: {:?}", turtle.pos());

            std::thread::sleep(std::time::Duration::from_millis(500));

            turtle.end_fill();
        });
}
