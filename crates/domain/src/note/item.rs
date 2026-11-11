use serde::{Deserialize, Serialize};

use super::Id;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub id: Id,
    pub text: String,
    #[serde(default)]
    pub done: bool,
}

impl Item {
    pub fn new(text: impl Into<String>) -> Self {
        Self { id: Id::new(), text: text.into(), done: false }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Patch {
    pub text: Option<String>,
    pub done: Option<bool>,
}

impl Patch {
    pub fn apply(self, i: &mut Item) {
        if let Some(v) = self.text {
            i.text = v;
        }
        if let Some(v) = self.done {
            i.done = v;
        }
    }
}
