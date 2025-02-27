use std::time::Duration;

use slowpoke::{SlowpokeLib, Turtle};

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("timed output")
        .run(|turtle| {
            turtle.teleport(-35, 130);
            turtle.speed(10);
            turtle.ontimer(time_tick, 1000);
        });
}

fn time_tick(turtle: &mut Turtle, _duration: Duration) {
    turtle.forward(70);
    turtle.right(30);
}
