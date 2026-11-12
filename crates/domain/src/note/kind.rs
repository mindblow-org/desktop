use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::{Color, Note, scene};
use crate::{Error, Result};

const TITLE: usize = 200;
const SHORT: usize = 500;
const LONG: usize = 200_000;
const ITEMS: usize = 1000;
const IMAGES: usize = 50;
const SCENE: usize = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Sticky,
    Text,
    Todo,
    Board,
}

pub trait Rule: Sync {
    fn check(&self, n: &Note) -> Result<()>;
}

struct Sticky;
struct Text;
struct Todo;
struct Board;

impl Kind {
    pub fn rule(self) -> &'static dyn Rule {
        match self {
            Kind::Sticky => &Sticky,
            Kind::Text => &Text,
            Kind::Todo => &Todo,
            Kind::Board => &Board,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Sticky => "sticky",
            Kind::Text => "text",
            Kind::Todo => "todo",
            Kind::Board => "board",
        }
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "sticky" => Ok(Kind::Sticky),
            "text" => Ok(Kind::Text),
            "todo" => Ok(Kind::Todo),
            "board" => Ok(Kind::Board),
            _ => Err(Error::Invalid("tipo desconhecido")),
        }
    }
}

fn fits(s: &str, max: usize, msg: &'static str) -> Result<()> {
    if s.chars().count() > max { Err(Error::Invalid(msg)) } else { Ok(()) }
}

fn deny(on: bool, msg: &'static str) -> Result<()> {
    if on { Err(Error::Invalid(msg)) } else { Ok(()) }
}

impl Rule for Sticky {
    fn check(&self, n: &Note) -> Result<()> {
        deny(!n.title.is_empty(), "sticky não tem título")?;
        deny(!n.items.is_empty(), "sticky não tem itens")?;
        deny(n.images.len() > 1, "sticky aceita uma imagem")?;
        fits(&n.body, SHORT, "sticky aceita até 500 caracteres")
    }
}

impl Rule for Text {
    fn check(&self, n: &Note) -> Result<()> {
        deny(!n.items.is_empty(), "texto não tem itens")?;
        deny(n.color != Color::Plain, "só o sticky tem cor")?;
        deny(n.images.len() > IMAGES, "nota aceita até 50 imagens")?;
        fits(&n.title, TITLE, "título aceita até 200 caracteres")?;
        fits(&n.body, LONG, "texto grande demais")
    }
}

impl Rule for Todo {
    fn check(&self, n: &Note) -> Result<()> {
        deny(!n.body.is_empty(), "todo não tem corpo, use itens")?;
        deny(n.color != Color::Plain, "só o sticky tem cor")?;
        deny(n.items.len() > ITEMS, "todo aceita até 1000 itens")?;
        deny(n.images.len() > IMAGES, "nota aceita até 50 imagens")?;
        fits(&n.title, TITLE, "título aceita até 200 caracteres")?;
        for i in &n.items {
            deny(i.text.trim().is_empty(), "item vazio")?;
            fits(&i.text, SHORT, "item aceita até 500 caracteres")?;
        }
        Ok(())
    }
}

impl Rule for Board {
    fn check(&self, n: &Note) -> Result<()> {
        deny(!n.items.is_empty(), "board não tem itens")?;
        deny(n.color != Color::Plain, "só o sticky tem cor")?;
        deny(n.images.len() > IMAGES, "nota aceita até 50 imagens")?;
        fits(&n.title, TITLE, "título aceita até 200 caracteres")?;
        deny(n.body.len() > SCENE, "board grande demais")?;
        deny(!n.body.is_empty() && scene::parse(&n.body).is_none(), "cena do board inválida")
    }
}
