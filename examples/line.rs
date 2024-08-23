use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.forward(100);
            println!("size before = {:?}", turtle.getscreensize());

            turtle.screensize([200, 100]);
            println!("size after = {:?}", turtle.getscreensize());

            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("size after = {:?}", turtle.getscreensize());
        });
}
