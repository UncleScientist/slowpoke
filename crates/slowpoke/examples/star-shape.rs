use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("star shape example from python docs")
        .run(|turtle| {
            turtle.speed("fastest");
            turtle.penup();
            turtle.setx(-100);
            turtle.pendown();
            turtle.pencolor((1., 0., 0.));
            turtle.fillcolor((1., 1., 0.));
            turtle.begin_fill();
            for _ in 0..36 {
                turtle.forward(200);
                turtle.left(170);
            }

            turtle.end_fill();
        });
}
