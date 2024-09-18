use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("polygon test")
        .run(|turtle| {
            println!("before: {:?}", turtle.getshapes());
            let shape =
                Shape::polygon(&[[100., 100.], [-100., 100.], [-100., -100.], [100., -100.]]);
            turtle.register_shape("different square", shape);
            println!(" after: {:?}", turtle.getshapes());
            for _ in 0..10 {
                turtle.shape("square");
                std::thread::sleep(std::time::Duration::from_millis(500));
                turtle.shape("different square");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}
