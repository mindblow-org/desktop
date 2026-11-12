use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Cor do papel do sticky, em pastel. `Plain` é o cinza de sempre, e vai como `""` no JSON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    #[default]
    #[serde(rename = "")]
    Plain,
    Yellow,
    Peach,
    Pink,
    Lilac,
    Sky,
    Mint,
}

impl Color {
    pub fn name(self) -> &'static str {
        match self {
            Color::Plain => "",
            Color::Yellow => "yellow",
            Color::Peach => "peach",
            Color::Pink => "pink",
            Color::Lilac => "lilac",
            Color::Sky => "sky",
            Color::Mint => "mint",
        }
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "" => Ok(Color::Plain),
            "yellow" => Ok(Color::Yellow),
            "peach" => Ok(Color::Peach),
            "pink" => Ok(Color::Pink),
            "lilac" => Ok(Color::Lilac),
            "sky" => Ok(Color::Sky),
            "mint" => Ok(Color::Mint),
            _ => Err(Error::Invalid("cor desconhecida")),
        }
    }
}
