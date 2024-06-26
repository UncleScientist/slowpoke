use rand::*;
use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("cool patterns")
        .run(|turtle| {
            turtle.speed("fastest");

            fn draw(turtle: &mut Turtle, n: isize, x: isize, angle: f64) {
                for i in 0..n {
                    let mut rng = rand::thread_rng();

                    let a: f64 = rng.gen::<f64>();
                    let b: f64 = rng.gen::<f64>();
                    let c: f64 = rng.gen::<f64>();

                    turtle.pencolor((a, b, c));
                    turtle.fillcolor((a, b, c));

                    turtle.begin_fill();
                    for _ in 0..5 {
                        turtle.forward((5 * n - 5 * i) as f64);
                        turtle.right(x as f64);
                        turtle.forward((5 * n - 5 * i) as f64);
                        turtle.right((72 - x) as f64);
                    }
                    turtle.end_fill();
                    turtle.right(angle);
                }
            }

            draw(turtle, 30, 144, 18.)
        });
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
