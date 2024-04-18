use slowpoke::{color_names::TurtleColor, *};

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("headings")
        .run(|turtle| {
            turtle.goto(90, 160);
            turtle.dot(None, TurtleColor::CurrentColor);

            turtle.goto(-50, 17);
            let h = turtle.towards(90, 160);

            println!("Turtle heading for (90, 160) = {h}");

            turtle.setheading(h);
            turtle.forward(200);

            let h = turtle.towards(0, 0);
            println!("Turtle heading for (0, 0) = {h}");
            turtle.setheading(h);
            turtle.forward(50);
        });
}
