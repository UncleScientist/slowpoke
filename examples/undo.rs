use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            const COUNT: usize = 10;
            for i in 0..COUNT {
                turtle.speed(i as u8);
                turtle.forward(50);
                turtle.left(87);
            }
            for i in 0..COUNT * 2 {
                turtle.speed(i as u8 / 2);
                turtle.undo();
            }
        });
}
