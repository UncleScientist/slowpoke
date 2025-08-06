use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(600, 600)
        .with_title("star shape example from python docs")
        .run(|turtle| {
            turtle.speed("fastest");
            turtle.penup();
            turtle.goto(280, 170);
            turtle.pendown();
            turtle.pencolor((1., 0., 0.));
            turtle.fillcolor((1., 1., 0.));
            while turtle.undobufferentries() > 0 {
                println!("entries: {}", turtle.undobufferentries());
                turtle.undo();
            }
        });
}
