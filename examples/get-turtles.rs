#![allow(clippy::cast_precision_loss)]

use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("lotsa turtles")
        .run(|turtle| {
            for _ in 0..11 {
                let _ = turtle.hatch();
            }

            let tlist = turtle.turtles();
            println!("There are {} turtles.", tlist.len());

            for (i, t) in turtle.turtles().iter_mut().enumerate() {
                t.right((i * 360 / 12) as f64);
                t.forward((i * 12) as f64);
            }

            loop {
                for t in &mut turtle.turtles() {
                    t.fillcolor("red");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                for t in &mut turtle.turtles() {
                    t.fillcolor("green");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                for t in &mut turtle.turtles() {
                    t.fillcolor("blue");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}
