use piston::Key;
use slowpoke::*;

fn main() {
    let ta = TurtleArgs::default();

    Turtle::run(&ta, |turtle| {
        turtle.onkey(left, Key::H);
        let mut t1 = turtle.hatch();
        t1.onkey(right, Key::J);

        turtle.onkey(forward20, Key::W);
        t1.onkey(forward20, Key::S);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

fn left(turtle: &mut Turtle, _key: Key) {
    turtle.left(90.);
}

fn right(turtle: &mut Turtle, _key: Key) {
    turtle.right(37);
}

fn forward20(turtle: &mut Turtle, _key: Key) {
    turtle.forward(20);
}
