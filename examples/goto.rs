use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("simple commands")
        .run(|turtle| {
            let mut s = String::new();

            turtle.goto(-100., -100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.goto(100., -100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.goto(100., 100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.goto(-100., 100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.setx(100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.sety(-100.);
            println!("press return");
            let _ = std::io::stdin().read_line(&mut s);

            turtle.bye();
        });
}
