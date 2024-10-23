use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            let name = turtle.textinput("This is a request...", "What is your name");
            turtle.forward(100);
            if let Some(name) = name {
                println!("your name is {name}");
            } else {
                println!("ok, anonymous");
            }
        });
}
