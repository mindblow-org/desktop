mod clock;
mod color;
mod draft;
mod id;
mod kind;
mod patch;
pub mod scene;
pub mod image;
pub mod item;

pub use clock::{Clock, merge};
pub use color::Color;
pub use draft::Draft;
pub use id::Id;
pub use image::Image;
pub use item::Item;
pub use kind::{Kind, Rule};
pub use patch::Patch;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: Id,
    pub kind: Kind,
    pub title: String,
    pub body: String,
    pub items: Vec<Item>,
    #[serde(default)]
    pub images: Vec<Image>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub color: Color,
    pub rank: f64,
    pub created: i64,
    pub updated: i64,
    /// Fica fora do JSON da nota: só o sync leva, em `Shot`.
    #[serde(skip)]
    pub clock: Clock,
}

impl Note {
    pub fn check(&self) -> Result<()> {
        self.kind.rule().check(self)
    }

    pub fn item(&mut self, id: Id) -> Result<&mut Item> {
        self.items.iter_mut().find(|i| i.id == id).ok_or(Error::NoItem)
    }
}
