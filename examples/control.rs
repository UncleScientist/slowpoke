use slowpoke::*;

fn main() {
    let ta = TurtleArgs::default();

    Turtle::run(&ta, |turtle| {
        turtle.onkey(left, 'h');
        let t1 = turtle.hatch();
        t1.onkey(right, 'j');

        turtle.onkey(forward20, 'w');
        t1.onkey(forward20, 's');

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

fn left(turtle: &mut Turtle, _key: char) {
    turtle.left(90.);
}

fn right(turtle: &mut Turtle, _key: char) {
    turtle.right(37);
}

fn forward20(turtle: &mut Turtle, _key: char) {
    turtle.forward(20);
}
