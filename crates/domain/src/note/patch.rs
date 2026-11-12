use serde::Deserialize;

use super::{Color, Item, Note};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Patch {
    pub title: Option<String>,
    pub body: Option<String>,
    pub items: Option<Vec<Item>>,
    pub pinned: Option<bool>,
    pub color: Option<Color>,
    pub rank: Option<f64>,
}

impl Patch {
    pub fn apply(self, n: &mut Note) {
        if let Some(v) = self.title {
            n.title = v;
        }
        if let Some(v) = self.body {
            n.body = v;
        }
        if let Some(v) = self.items {
            n.items = v;
        }
        if let Some(v) = self.pinned {
            n.pinned = v;
        }
        if let Some(v) = self.color {
            n.color = v;
        }
        if let Some(v) = self.rank {
            n.rank = v;
        }
    }
}
