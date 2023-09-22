use slowpoke::*;

fn main() {
    Turtle::start(800, 800, |turtle| {
        println!("starting at {:?}", turtle.pos());

        turtle.forward(100.);
        println!("pos: {:?}", turtle.pos());

        turtle.right(90.);
        turtle.penup();

        turtle.forward(100.);
        println!("pos: {:?}", turtle.pos());

        turtle.right(90.);
        turtle.pendown();

        turtle.forward(100.);
        println!("pos: {:?}", turtle.pos());

        turtle.home();
        println!("pos: {:?}", turtle.pos());

        turtle.forward(100.);
        println!("pos: {:?}", turtle.pos());

        turtle.right(90.);
        turtle.backward(100.);

        println!("press return to continue");
        let mut s = String::new();
        let _ = std::io::stdin().read_line(&mut s);

        turtle.clearscreen();
        _spiky_fractal(turtle, 3, 243.);

        // turtle.right(90.);
        // _square_fractal(turtle, 3, 243.);
    });
}

fn _spiky_fractal(turtle: &mut Turtle, order: usize, length: f64) {
    if order == 0 {
        turtle.forward(length);
    } else {
        _spiky_fractal(turtle, order - 1, length / 3.);
        turtle.left(60.);
        _spiky_fractal(turtle, order - 1, length / 3.);
        turtle.right(120.);
        _spiky_fractal(turtle, order - 1, length / 3.);
        turtle.left(60.);
        _spiky_fractal(turtle, order - 1, length / 3.);
    }
}

fn _square_fractal(turtle: &mut Turtle, order: usize, length: f64) {
    if order == 0 {
        turtle.forward(length);
    } else {
        _square_fractal(turtle, order - 1, length / 3.);
        turtle.left(90.);
        _square_fractal(turtle, order - 1, length / 3.);
        turtle.right(90.);
        _square_fractal(turtle, order - 1, length / 3.);
        turtle.right(90.);
        _square_fractal(turtle, order - 1, length / 3.);
        turtle.left(90.);
        _square_fractal(turtle, order - 1, length / 3.);
    }
}
