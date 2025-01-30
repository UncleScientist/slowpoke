use slowpoke::{Slowpoke, Turtle};

fn main() {
    Slowpoke::default()
        .with_size(400, 400)
        .with_title("a line")
        .run(|turtle| {
            turtle.onclick(draw_line_to);
            turtle.ondrag(|_turtle, _x, _y| {} /*println!("drag: {x},{y}")*/);
            turtle.onrelease(change_bg_color);
        });
}

fn draw_line_to(turtle: &mut Turtle, x: f32, y: f32) {
    turtle.goto(x, y);
}

fn change_bg_color(turtle: &mut Turtle, x: f32, y: f32) {
    let size = turtle.getscreensize();
    let red = (x + size[0] as f32 / 2.) / size[0] as f32;
    let blue = (y + size[1] as f32 / 2.) / size[1] as f32;
    turtle.write(&format!(
        "w{}, h{}, x{x}, y{y}, r{red}, b{blue}",
        size[0], size[1]
    ));
    turtle.bgcolor((red, blue, 0.5));
}
