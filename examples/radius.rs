use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(600, 400)
        .with_title("a corner radius")
        .run(|turtle| {
            let mut s = String::new();

            turtle.speed(1);
            turtle.fillcolor("#e70010");
            turtle.begin_fill();
            turtle.setheading(0);
            println!("forward 200");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.forward(200);

            println!("circle -5 90");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.circle(-5).with_extent(90);

            println!("forward 10");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.forward(10);

            println!("circle -5 90");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.circle(-5).with_extent(90);

            println!("forward 207");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.forward(207);

            println!("circle -5 90");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.circle(-5).with_extent(90);

            println!("forward 10");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.forward(10);

            println!("circle -5 90");
            let _ = std::io::stdin().read_line(&mut s);
            turtle.circle(-5).with_extent(90);
            /*
            turtle.forward(100);
            turtle.circle(-30, 90, 20);
            turtle.forward(100);
                */
        });
}
