use slowpoke::{Slowpoke, Turtle};

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.onclick(draw_line_to);
            turtle.ondrag(|_turtle, _x, _y| {} /*println!("drag: {x},{y}")*/);
            turtle.onrelease(|_turtle, _x, _y| {} /*println!("release: {x},{y}")*/);
        });
}

fn draw_line_to(turtle: &mut Turtle, x: f32, y: f32) {
    turtle.goto(x, y);
}
