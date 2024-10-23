use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("nested cubes")
        .run(|turtle| {
            turtle.speed("fastest");
            turtle.width(2);
            for i in 0..290 {
                let i = i as f64;
                turtle.pencolor(((i / 300.0 * 4.0) % 1.0, 1.0, 1.0));
                turtle.forward(i);
                turtle.right(60.0);
            }
        });
}
