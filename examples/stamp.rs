use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a square stamp")
        .run(|turtle| {
            let mut v = Vec::new();
            for _ in 0..4 {
                println!("forward");
                turtle.forward(100);
                println!("right");
                turtle.right(90);
                println!("stamp");
                v.push(turtle.stamp());
            }

            turtle.teleport(50, 50);

            println!("{v:?}");

            for id in v {
                println!("clearing stamp {id}");
                turtle.clearstamp(id);
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
}
