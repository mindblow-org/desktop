use serde::{Deserialize, Serialize};

use super::Id;
use crate::{Error, Result};

pub const MAX: usize = 20 * 1024 * 1024;
const TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/webp", "image/gif"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub id: Id,
    pub mime: String,
    pub size: u64,
    #[serde(default)]
    pub w: u32,
    #[serde(default)]
    pub h: u32,
}

impl Image {
    pub fn new(mime: &str, size: usize, w: u32, h: u32) -> Result<Self> {
        if !TYPES.contains(&mime) {
            return Err(Error::Invalid("tipo de imagem não aceito: use png, jpeg, webp ou gif"));
        }
        if size == 0 {
            return Err(Error::Invalid("imagem vazia"));
        }
        if size > MAX {
            return Err(Error::Invalid("imagem passa de 20 MB"));
        }
        Ok(Self { id: Id::new(), mime: mime.into(), size: size as u64, w, h })
    }
}
