use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(500, 500)
        .with_title("clearing stamps")
        .run(|turtle| {
            for _ in 0..8 {
                turtle.stamp();
                turtle.forward(30);
            }
            std::thread::sleep(std::time::Duration::from_millis(500));

            println!("clearing first two");
            turtle.clearstamps(2);
            std::thread::sleep(std::time::Duration::from_millis(500));

            println!("clearing last two");
            turtle.clearstamps(-2);
            std::thread::sleep(std::time::Duration::from_millis(500));

            println!("clearing remaining");
            turtle.clearstamps(0);
        });
}
