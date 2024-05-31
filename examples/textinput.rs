use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            let name = turtle.textinput("Title", "What is your name");
            turtle.forward(100);
            println!("your name is {name}");
        });
}
