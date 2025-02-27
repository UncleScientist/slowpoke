use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("flashing turtle")
        .run(|turtle| {
            for _ in 0..5 {
                turtle.fillcolor("red");
                std::thread::sleep(std::time::Duration::from_millis(500));
                turtle.fillcolor("blue");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}
