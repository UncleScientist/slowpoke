use slowpoke::*;

fn main() {
    Turtle::start(|turtle| {
        turtle.forward(100.);
        turtle.right(90.);
        turtle.penup();
        turtle.forward(100.);
        turtle.right(90.);
        turtle.pendown();
        turtle.forward(100.);

        turtle.goto(0., 0.);
        turtle.forward(100.);

        // _spiky_fractal(turtle, 3, 243.);
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
