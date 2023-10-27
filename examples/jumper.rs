use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("simple commands")
        .run(|turtle| {
            turtle.penup();
            turtle.goto(-100., -100.);
            turtle.pendown();
            turtle.goto(100., -100.);
            turtle.goto(100., 100.);
            turtle.goto(-100., 100.);
            turtle.sety(50.);
            turtle.setx(-50.);
            turtle.home();
        });
}
