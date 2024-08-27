use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.forward(100); // draw right
            turtle.right(90); // point down
            turtle.forward(50); // draw down
            turtle.undo(); // back up 50 pixels
            turtle.undo(); // turn to the left by 90
            turtle.backward(100); // return to origin
            std::process::exit(0);
        });
}
