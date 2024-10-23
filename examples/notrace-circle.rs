use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(500, 850)
        .with_title("Circles")
        .run(|turtle| {
            turtle.pencolor("white");
            turtle.bgcolor("grey");
            turtle.speed(1);
            turtle.tracer(false);
            for i in 3..20 {
                turtle.circle(10 + i as u32 * 10).with_steps(i);
            }
            for i in 3..20 {
                turtle.circle(-(10. + i as f64 * 10.)).with_steps(i);
            }
            turtle.tracer(true);
        });
}
