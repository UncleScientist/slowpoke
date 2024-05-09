use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(600, 600)
        .with_title("star shape example from python docs")
        .run(|turtle| {
            turtle.speed("fastest");
            for (cx, cy) in [(-280, 180), (80, 180), (-280, -180), (80, -180)] {
                turtle.penup();
                turtle.goto(cx, cy);
                turtle.pendown();
                turtle.pencolor((1., 0., 0.));
                turtle.fillcolor((1., 1., 0.));
                turtle.begin_fill();
                for _ in 0..36 {
                    turtle.forward(200);
                    turtle.left(170);
                }
                turtle.end_fill();
            }
            while turtle.undobufferentries() > 0 {
                turtle.undo();
            }
        });
}
