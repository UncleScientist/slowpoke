use rand::Rng;
use slowpoke::{SlowpokeLib, Turtle};

fn main() {
    SlowpokeLib::default()
        .with_size(500, 500)
        .with_title("cool patterns")
        .run(|turtle| {
            fn draw(turtle: &mut Turtle, count: isize, x: isize, angle: f64) {
                for i in 0..count {
                    let mut rng = rand::thread_rng();

                    let a: f64 = rng.gen::<f64>();
                    let b: f64 = rng.gen::<f64>();
                    let c: f64 = rng.gen::<f64>();

                    turtle.pencolor((a, b, c));
                    turtle.fillcolor((a, b, c));

                    turtle.begin_fill();
                    for _ in 0..5 {
                        turtle.forward(to_f64(5 * count - 5 * i));
                        turtle.right(to_f64(x));
                        turtle.forward(to_f64(5 * count - 5 * i));
                        turtle.right(to_f64(72 - x));
                    }
                    turtle.end_fill();
                    turtle.right(angle);
                }
            }

            turtle.speed("fastest");
            draw(turtle, 30, 144, 18.);
        });
}

#[allow(clippy::cast_precision_loss)]
fn to_f64(val: isize) -> f64 {
    val as f64
}

/*
from turtle import *

import random

speed(speed ='fastest')

def draw(n, x, angle):
    # loop for number of stars
    for i in range(n):

        colormode(255)

        # choosing random integers
        # between 0 and 255
        # to generate random rgb values
        a = random.randint(0, 255)
        b = random.randint(0, 255)
        c = random.randint(0, 255)

        # setting the outline
        # and fill colour
        pencolor(a, b, c)
        fillcolor(a, b, c)

        # begins filling the star
        begin_fill()

        # loop for drawing each star
        for j in range(5):

            forward(5 * n-5 * i)
            right(x)
            forward(5 * n-5 * i)
            right(72 - x)

        # colour filling complete
        end_fill()

        # rotating for
        # the next star
        rt(angle)


# setting the parameters
n = 30    # number of stars
x = 144   # exterior angle of each star
angle = 18    # angle of rotation for the spiral

draw(n, x, angle)
*/
