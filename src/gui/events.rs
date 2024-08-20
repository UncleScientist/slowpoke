pub(crate) enum TurtleEvent {
    WindowResize(u32, u32), // width, height
    KeyPress(char),
    KeyRelease(char),
    MousePress(u32, u32, u32), // button, click-x, click-y
    Timer,
    Unhandled, // TODO: remove this, and implment TryFrom<Iced::Event> for Self
}
