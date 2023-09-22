use slowpoke::*;

fn main() {
    Turtle::start(|turtle| {
        spiky_fractal(turtle, 3, 243.);
        turtle.right(90.);
        square_fractal(turtle, 3, 243.);
    });
}

fn spiky_fractal(turtle: &mut Turtle, order: usize, length: f64) {
    if order == 0 {
        turtle.forward(length);
    } else {
        spiky_fractal(turtle, order - 1, length / 3.);
        turtle.left(60.);
        spiky_fractal(turtle, order - 1, length / 3.);
        turtle.right(120.);
        spiky_fractal(turtle, order - 1, length / 3.);
        turtle.left(60.);
        spiky_fractal(turtle, order - 1, length / 3.);
    }
}

fn square_fractal(turtle: &mut Turtle, order: usize, length: f64) {
    if order == 0 {
        turtle.forward(length);
    } else {
        square_fractal(turtle, order - 1, length / 3.);
        turtle.left(90.);
        square_fractal(turtle, order - 1, length / 3.);
        turtle.right(90.);
        square_fractal(turtle, order - 1, length / 3.);
        turtle.right(90.);
        square_fractal(turtle, order - 1, length / 3.);
        turtle.left(90.);
        square_fractal(turtle, order - 1, length / 3.);
    }
}
