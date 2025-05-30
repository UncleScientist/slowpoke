use rand::Rng;
use slowpoke::Slowpoke;

fn main() {
    Slowpoke::default()
        .with_size(500, 500)
        .with_title("cool patterns")
        .run(|turtle| {
            let mut rng = rand::rng();

            for _ in 0..10 {
                let visible = if rng.random::<bool>() {
                    println!("hiding");
                    turtle.hideturtle();
                    false
                } else {
                    println!("showing");
                    turtle.showturtle();
                    true
                };
                println!("we think '{visible}', gui says: '{}'", turtle.isvisible());
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}
