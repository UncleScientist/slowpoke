use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("lotsa turtles")
        .run(|turtle| {
            for _ in 0..11 {
                let _ = turtle.hatch();
            }

            let tlist = turtle.turtles();
            println!("There are {} turtles.", tlist.len());

            for (i, t) in turtle.turtles().iter_mut().enumerate() {
                t.right(i as u32 * 360 / 12);
                t.forward(i as u32 * 12);
            }

            loop {
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("red");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("green");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                for t in turtle.turtles().iter_mut() {
                    t.fillcolor("blue");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}
