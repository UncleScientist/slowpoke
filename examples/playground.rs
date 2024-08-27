use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("simple commands")
        .run(|turtle| {
            turtle.speed(5);
            turtle.bgcolor("grey");
            turtle.dot();
            turtle.dot().with_color("blue");
            turtle.right(0);
            println!(
                "starting at {:?}, heading of {:?}",
                turtle.pos(),
                turtle.heading()
            );

            println!(
                "Current turtle shape: {}",
                turtle.shape(TurtleShapeName::GetCurrent)
            );

            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.penup();

            turtle.fd(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.pendown();

            turtle.pencolor((0.5, 0.8, 0.4));
            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.shape("arrow");

            turtle.home();
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.penwidth(5);
            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.dot().with_size(20).with_color((0.8, 0.4, 0.5));

            turtle.right(90);
            turtle.backward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.setx(-175);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.dot().with_size(10);

            println!(
                "Final turtle shape: {}",
                turtle.shape(TurtleShapeName::GetCurrent)
            );

            println!("press return to finish");
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);
            std::process::exit(0);
        });
}
