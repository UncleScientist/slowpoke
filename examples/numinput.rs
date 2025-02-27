use clamp_to::Clamp;
use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("Input a number")
        .run(|turtle| {
            turtle.bgcolor("grey");
            let num = turtle.numinput("This is a request...", "How many sides?");
            if let Some(num) = num {
                let num = num.clamp_to_usize();
                if num > 2 && num < 200 {
                    turtle.penup();
                    turtle.goto(0, -150);
                    turtle.pendown();
                    turtle.circle(150).with_steps(num);
                    turtle.teleport(0, 0);
                    turtle.setheading(0);
                } else {
                    println!("{num} is not in the range 3..200");
                }
            } else {
                println!("cancelled input");
            }
        });
}
