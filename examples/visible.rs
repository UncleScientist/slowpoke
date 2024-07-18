use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("an invisible circle-drawerer")
        .run(|turtle| {
            turtle.hideturtle();
            turtle.circle(150);
            turtle.showturtle();
        });
}
