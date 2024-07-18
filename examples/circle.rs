use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            turtle.pencolor("black");
            turtle.bgcolor("grey");
            turtle.speed(1);
            turtle.steps(5).circle(50);
            for i in 3..20 {
                turtle.extent(360).steps(i).circle(10 + i as u32 * 10);
            }
            for i in 3..20 {
                turtle.extent(360).steps(i).circle(-(10. + i as f64 * 10.));
            }
        });
}
