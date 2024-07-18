use slowpoke::*;

fn main() {
    TurtleArgs::default()
        .with_size(400, 400)
        .with_title("heading demo")
        .run(|turtle| {
            turtle.speed(1);
            turtle.dot().with_size(3).with_color("red");
            turtle.teleport(10, 20);
            println!("teleported to 10,20; pos = {:?}", turtle.pos());

            // draw to the right
            turtle.setheading(0);
            turtle.forward(100);
            println!("heading 0; forward 100; pos = {:?}", turtle.pos());

            turtle.teleport(10, 20);
            println!("teleported to 10,20; pos = {:?}", turtle.pos());

            // draw "south" or down
            turtle.setheading(270);
            turtle.forward(100);
            println!("heading 270; forward 100; pos = {:?}", turtle.pos());

            turtle.teleport(10, 20);
            println!("teleported to 10,20; pos = {:?}", turtle.pos());

            // draw "north" or up
            turtle.setheading(90);
            turtle.forward(100);
            println!("pos = {:?}", turtle.pos());
            turtle.teleport(10, 20);

            // draw "west" or to the left
            turtle.setheading(180);
            turtle.forward(100);
            println!("pos = {:?}", turtle.pos());
            turtle.teleport(10, 20);

            turtle.left(30);
        });
}
