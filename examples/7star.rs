use slowpoke::{SlowpokeLib, Turtle};

fn main() {
    SlowpokeLib::default()
        .with_size(400, 400)
        .with_title("4 7-pointed stars [Press C to change size]")
        .run(|turtle| {
            turtle.speed(10);
            turtle.tracer(false);
            turtle.onkeypress(sizer, 'c');
            for pos in [(50, 130), (-150, 130), (50, -80), (-150, -80)] {
                turtle.teleport(pos.0, pos.1);
                for _ in 0..7 {
                    turtle.forward(100);
                    turtle.right(2. * 360. / 7.);
                }
            }
        });
}

fn sizer(turtle: &mut Turtle, _key: char) {
    let new_size = turtle.numinput("New Size", "Enter a new width/height for the window");
    if let Some(ns) = new_size {
        turtle.screensize([ns as isize, ns as isize]);
    }
}
