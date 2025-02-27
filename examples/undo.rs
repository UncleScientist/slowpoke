use slowpoke::SlowpokeLib;
use std::io::Write;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            for i in 0..17 {
                turtle.speed(i);
                turtle.forward(50);
                turtle.left(87);
            }

            let entries = turtle.undobufferentries();
            println!("There are {entries} steps to undo");
            for i in 0..entries {
                turtle.speed(i % 10 + 1);
                print!(".. {}", i + 1);
                let _ = std::io::stdout().flush();
                turtle.undo();
            }
            println!();
        });
}
