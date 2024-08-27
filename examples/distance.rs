use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a distance")
        .run(|turtle| {
            println!("distance to 0, 100: {}", turtle.distance((0, 100)));

            let mut other_turtle = turtle.hatch();
            other_turtle.goto(50, 50);
            println!(
                "distance from (0, 0) to other turtle @ (50, 50): {}",
                turtle.distance(&other_turtle)
            );
        });
}
