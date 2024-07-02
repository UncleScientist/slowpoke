use crate::Response;

#[derive(Default)]
enum PopupType {
    TextInput {
        turtle_id: u64,
        which: usize,
        prompt: String,
        text_input_field: String,
    },
    NumericalInput {
        turtle_id: u64,
        which: usize,
        prompt: String,
        text_input_field: String,
    },
    ErrorMessage(String),
    #[default]
    Null,
}

impl PopupType {
    fn id(&self) -> u64 {
        match self {
            PopupType::TextInput { turtle_id, .. }
            | PopupType::NumericalInput { turtle_id, .. } => *turtle_id,
            _ => panic!("invalid popup for turtle id"),
        }
    }

    fn which(&self) -> usize {
        match self {
            PopupType::TextInput { which, .. } | PopupType::NumericalInput { which, .. } => *which,
            _ => panic!("invalid popup for turtle id"),
        }
    }

    fn prompt(&self) -> String {
        match self {
            PopupType::TextInput { prompt, .. } | PopupType::NumericalInput { prompt, .. } => {
                prompt.clone()
            }
            _ => panic!("invalid popup for turtle id"),
        }
    }

    fn text_input_field(&self) -> String {
        match self {
            PopupType::TextInput {
                text_input_field, ..
            }
            | PopupType::NumericalInput {
                text_input_field, ..
            } => text_input_field.clone(),
            _ => panic!("invalid popup for turtle id"),
        }
    }
}

#[derive(Default)]
pub(super) struct PopupData {
    title: String,
    popup: PopupType,
}

impl PopupData {
    pub fn mainwin(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Self::default()
        }
    }

    pub fn text_input(title: &str, prompt: &str, turtle_id: u64, which: usize) -> Self {
        Self {
            title: title.to_string(),
            popup: PopupType::TextInput {
                prompt: prompt.to_string(),
                turtle_id,
                which,
                text_input_field: "".to_string(),
            },
        }
    }

    pub fn num_input(title: &str, prompt: &str, turtle_id: u64, which: usize) -> Self {
        Self {
            title: title.to_string(),
            popup: PopupType::NumericalInput {
                prompt: prompt.to_string(),
                turtle_id,
                which,
                text_input_field: "".to_string(),
            },
        }
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn set_message<T: Into<String>>(&mut self, message: T) {
        match &mut self.popup {
            PopupType::TextInput {
                text_input_field, ..
            } => *text_input_field = message.into(),
            PopupType::NumericalInput {
                text_input_field, ..
            } => *text_input_field = message.into(),
            _ => panic!("invalid popup type for message"),
        }
    }

    pub fn id(&self) -> u64 {
        self.popup.id()
    }

    pub fn which(&self) -> usize {
        self.popup.which()
    }

    pub(crate) fn prompt(&self) -> String {
        self.popup.prompt()
    }

    pub(crate) fn get_text(&self) -> String {
        self.popup.text_input_field()
    }

    pub(crate) fn get_response(&self) -> Result<Response, String> {
        match &self.popup {
            PopupType::TextInput {
                text_input_field, ..
            } => Ok(Response::TextInput(text_input_field.clone())),
            PopupType::NumericalInput {
                text_input_field, ..
            } => {
                if let Ok(val) = text_input_field.parse::<f32>() {
                    Ok(Response::NumInput(val))
                } else {
                    Err("Not a floating point value".to_string())
                }
            }
            _ => panic!("invalid window type for retriving data"),
        }
    }

    pub(crate) fn error_message(message: &str) -> PopupData {
        PopupData {
            title: "".to_string(),
            popup: PopupType::ErrorMessage(message.to_string()),
        }
    }

    pub(crate) fn get_error(&self) -> Option<String> {
        if let PopupType::ErrorMessage(msg) = &self.popup {
            Some(msg.clone())
        } else {
            None
        }
    }
}
