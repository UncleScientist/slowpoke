use slowpoke::TurtleShapeName;
use slowpoke_iced::Slowpoke;

fn main() {
    Slowpoke::default()
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
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pen is down? {}\ncalling penup()", turtle.isdown());
            turtle.penup();
            println!("pen is down? {}", turtle.isdown());

            turtle.fd(100);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.right(90);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("calling pendown()");
            turtle.pendown();
            println!("pen is down? {}", turtle.isdown());

            turtle.pencolor((0.5, 0.8, 0.4));
            turtle.forward(100);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.shape("arrow");

            turtle.home();
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.width(5);
            turtle.forward(100);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.dot().with_size(20).with_color((0.8, 0.4, 0.5));

            turtle.right(90);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            turtle.backward(100);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.setx(-175);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));
            println!("pos: {:?}, heading: {:?}", turtle.pos(), turtle.heading());

            turtle.dot().with_size(10);

            println!(
                "Final turtle shape: {}",
                turtle.shape(TurtleShapeName::GetCurrent)
            );

            println!("press return to clear");
            let mut s = String::new();
            let _ = std::io::stdin().read_line(&mut s);

            turtle.clearscreen();

            println!("press return to move forward");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.forward(100);
            turtle.title(format!("{:?} @ {} deg", turtle.pos(), turtle.heading()));

            println!("press return to exit");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.bye();
        });
}
