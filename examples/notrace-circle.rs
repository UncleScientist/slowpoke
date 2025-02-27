use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(500, 850)
        .with_title("Circles")
        .run(|turtle| {
            turtle.pencolor("white");
            turtle.bgcolor("grey");
            turtle.speed(1);
            turtle.tracer(false);
            for i in 3i16..20 {
                turtle.circle(10 + i * 10).with_steps(i.unsigned_abs());
            }
            for i in 3i16..20 {
                turtle.circle(-(10 + i * 10)).with_steps(i.unsigned_abs());
            }
            turtle.tracer(true);
        });
}
