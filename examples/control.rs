use piston::Key;
use slowpoke::*;

fn main() {
    Turtle::start(800, 800, |turtle| {
        turtle.onkey(left, Key::H);

        loop {
            turtle.forward(1.);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

fn left(turtle: &mut Turtle, _key: Key) {
    println!("got key {_key:?}");
    turtle.left(90.);
    println!("moved left");
    turtle.forward(20.);
    println!("moved forward");
}
