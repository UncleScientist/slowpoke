#[derive(Debug)]
pub enum TurtleEvent {
    WindowResize(isize, isize), // width, height
    KeyPress(char),
    KeyRelease(char),
    MousePosition(f32, f32), // x and y
    MousePress(f32, f32),    // click-x, click-y
    MouseRelease(f32, f32),  // click-x, click-y
    MouseDrag(f32, f32),     // x and y
    _Timer,
    Unhandled, // TODO: remove this, and implement TryFrom<Iced::Event> for Self
}
