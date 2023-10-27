use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            for i in 3..20 {
                turtle.circle(10 + i as u32 * 10, 360., i);
            }
        });
}
