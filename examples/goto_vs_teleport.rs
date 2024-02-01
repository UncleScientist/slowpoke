use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("Goto Vs Teleport")
        .run(|turtle| {
            let mut s = String::new();

            turtle.goto(-100., -100.);
            println!("press return to teleport");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.teleport(100., -100.);
            println!("press return to goto");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.goto(100., 100.);
            println!("press return to teleport");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.teleport(-100., 100.);
            println!("press return to goto");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.goto(200, 30);
            println!("press return to teleport");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.teleport(-50, -50);
            println!("press return to exit");
            let _ = std::io::stdin().read_line(&mut s);

            std::process::exit(0);
        });
}
