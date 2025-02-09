use slowpoke::{Slowpoke, Turtle};

fn main() {
    Slowpoke::new()
        .with_size(400, 400)
        .with_title("A Spikey Fractal")
        .run(|turtle| {
            turtle.speed(10);
            turtle.penup();
            turtle.goto(-243. / 2., 243. / 2.);
            turtle.pendown();
            for _ in 0..4 {
                spiky_fractal(turtle, 3, 243.);
                turtle.right(90.);
            }
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

/*
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
*/
