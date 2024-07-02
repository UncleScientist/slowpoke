#[derive(Default)]
pub(super) struct PopupData {
    title: String,
    turtle_id: u64,
    which: usize,
    prompt: String,
    text_input_field: String,
}

impl PopupData {
    pub fn mainwin(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }

    pub fn new(title: &str, prompt: &str, turtle_id: u64, which: usize) -> Self {
        Self {
            title: title.to_string(),
            prompt: prompt.to_string(),
            turtle_id,
            which,
            text_input_field: "".to_string(),
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn set_message<T: Into<String>>(&mut self, message: T) {
        self.text_input_field = message.into()
    }

    pub fn id(&self) -> u64 {
        self.turtle_id
    }

    pub fn which(&self) -> usize {
        self.which
    }

    pub(crate) fn prompt(&self) -> String {
        self.prompt.clone()
    }

    pub(crate) fn get_text(&self) -> String {
        self.text_input_field.clone()
    }
}
