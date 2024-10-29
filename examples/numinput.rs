use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("Input a number")
        .run(|turtle| {
            turtle.bgcolor("grey");
            let num = turtle.numinput("This is a request...", "Gimmie a floating point number");
            if let Some(num) = num {
                println!("double your number is {}", num * 2.);
            } else {
                println!("you cancelled the dialogue box");
            }
        });
}
