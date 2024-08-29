#[derive(Debug)]
pub(crate) enum TurtleEvent {
    WindowResize(u32, u32), // width, height
    KeyPress(char),
    KeyRelease(char),
    MousePosition(f32, f32), // x and y
    MousePress(f32, f32),    // click-x, click-y
    MouseRelease(f32, f32),  // click-x, click-y
    MouseDrag(f32, f32),     // x and y
    _Timer,
    Unhandled, // TODO: remove this, and implment TryFrom<Iced::Event> for Self
}
