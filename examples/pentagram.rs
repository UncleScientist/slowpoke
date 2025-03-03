use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("a pentagram")
        .run(|turtle| {
            turtle.fillcolor("green");
            turtle.begin_fill();
            for _ in 0..5 {
                turtle.forward(100);
                turtle.right(180 - 36);
            }
            turtle.end_fill();
            turtle.speed(1);
            turtle.goto(-200, -200);
            while turtle.undobufferentries() > 0 {
                turtle.undo();
            }
        });
}
