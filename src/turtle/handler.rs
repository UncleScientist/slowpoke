use std::collections::HashMap;

use crate::gui::popup::PopupData;

use super::{types::PopupID, TurtleID};

pub(crate) struct Handler<T, I: Resize + SetBackgroundColor> {
    pub(crate) last_id: TurtleID,
    pub(crate) turtle: HashMap<TurtleID, T>,
    pub(crate) popups: HashMap<PopupID, PopupData>,
    pub(crate) title: String,
    pub(crate) internal: I,
}

pub(crate) trait Resize {}
pub(crate) trait SetBackgroundColor {}
