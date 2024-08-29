use slowpoke::*;

fn main() {
    let ta = TurtleArgs::default();

    Turtle::run(&ta, |turtle| {
        let mut t1 = turtle.hatch();

        turtle.speed(1);
        t1.speed(1);

        turtle.fillcolor("red");
        t1.fillcolor("blue");

        turtle.onkeypress(left, 'h');
        t1.onkeypress(right, 'j');

        turtle.onkeypress(forward10, 'w');
        t1.onkeypress(forward30, 's');

        turtle.onkeyrelease(right, 'i');
        t1.onkeyrelease(left, 'o');

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

fn forward30(turtle: &mut Turtle, _key: char) {
    turtle.forward(30);
}

fn forward10(turtle: &mut Turtle, _key: char) {
    turtle.forward(10);
}
