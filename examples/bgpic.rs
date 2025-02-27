use slowpoke::SlowpokeLib;

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("a background picture")
        .run(|turtle| {
            turtle.bgpic("assets/turtle-star.png");
        });
}
