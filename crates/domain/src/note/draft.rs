use serde::Deserialize;

use super::{Clock, Id, Item, Kind, Note};

#[derive(Clone, Debug, Deserialize)]
pub struct Draft {
    pub kind: Kind,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub pinned: bool,
}

impl Draft {
    pub fn new(kind: Kind) -> Self {
        Self { kind, title: String::new(), body: String::new(), items: Vec::new(), pinned: false }
    }

    pub fn title(mut self, v: impl Into<String>) -> Self {
        self.title = v.into();
        self
    }

    pub fn body(mut self, v: impl Into<String>) -> Self {
        self.body = v.into();
        self
    }

    pub fn item(mut self, v: impl Into<String>) -> Self {
        self.items.push(Item::new(v));
        self
    }

    pub fn pin(mut self) -> Self {
        self.pinned = true;
        self
    }

    pub fn note(self, rank: f64, now: i64) -> Note {
        let mut n = Note {
            id: Id::new(),
            kind: self.kind,
            title: self.title,
            body: self.body,
            items: self.items,
            images: Vec::new(),
            pinned: self.pinned,
            rank,
            created: now,
            updated: now,
            clock: Clock::default(),
        };
        n.clock = Clock::at(&n, now);
        n
    }
}
