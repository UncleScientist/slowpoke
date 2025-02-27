use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("an octagon")
        .run(|turtle| {
            let dist = 8;
            turtle.left(90);
            turtle.forward(dist / 2);
            println!("{:?}", turtle.pos());
            for _ in 0..7 {
                turtle.left(180 - 135);
                turtle.forward(dist);
                println!("{:?}", turtle.pos());
            }
            turtle.left(180 - 135);
            turtle.forward(dist / 2);
        });
}
