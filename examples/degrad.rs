use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("degrees and radians")
        .run(|turtle| {
            turtle.right(90);
            println!("        degrees: {}", turtle.heading());
            turtle.radians();
            println!("        radians: {}", turtle.heading());
            turtle.degrees(400);
            println!("       gradians: {}", turtle.heading());
            turtle.degrees(360);
            println!("back to degrees: {}", turtle.heading());
        });
}
