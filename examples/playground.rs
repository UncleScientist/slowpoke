use slowpoke::{color_names::CurrentColor, *};

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("simple commands")
        .run(|turtle| {
            turtle.bgcolor("grey");
            turtle.dot(None, Some(CurrentColor));
            turtle.dot(None, Some("blue"));
            turtle.right(0);
            println!(
                "starting at {:?}, heading of {:?}",
                turtle.pos(),
                turtle.heading()
            );

            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.penup();

            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.pendown();

            turtle.pencolor((0.5, 0.8, 0.4));
            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.home();
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.penwidth(5);
            turtle.forward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.dot(Some(20.), Some((0.8, 0.4, 0.5)));

            turtle.right(90);
            turtle.backward(100);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.setx(-175);
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            println!("press return to finish");
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);
            std::process::exit(0);
        });
}
