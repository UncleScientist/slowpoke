use slowpoke::{Slowpoke, Turtle};

fn main() {
    Slowpoke::new()
        .with_size(500, 500)
        .with_title("A Squarish Fractal")
        .run(|turtle| {
            turtle.tracer(false);
            turtle.penup();
            turtle.goto(-243. / 2., 243. / 2.);
            turtle.pendown();
            for _ in 0..4 {
                square_fractal(turtle, 4, 243.);
                turtle.right(90.);
            }
            turtle.tracer(true);
        });
}

fn square_fractal(turtle: &mut Turtle, order: usize, length: f64) {
    if order == 0 {
        turtle.forward(length);
    } else {
        if order == 3 {
            println!("pos = {:?}", turtle.pos());
        }
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
