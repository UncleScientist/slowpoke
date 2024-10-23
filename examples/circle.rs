use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(800, 800)
        .with_title("Circles")
        .run(|turtle| {
            turtle.pencolor("black");
            turtle.bgcolor("grey");
            turtle.speed(1);
            turtle.fillcolor("light green");
            for i in 3..20 {
                turtle.circle(10 + i as u32 * 10).with_steps(i);
            }
            for i in 3..20 {
                turtle.circle(-(10. + i as f64 * 10.)).with_steps(i);
            }
        });
}
