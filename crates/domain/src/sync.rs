use serde::{Deserialize, Serialize};

use crate::{Clock, Id, Note, Result};

/// Uma nota como ela anda entre aparelhos: com o relógio de cada campo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    pub note: Note,
    pub clock: Clock,
}

impl From<Note> for Shot {
    fn from(mut n: Note) -> Self {
        let clock = std::mem::take(&mut n.clock);
        Self { note: n, clock }
    }
}

impl From<Shot> for Note {
    fn from(s: Shot) -> Self {
        Note { clock: s.clock, ..s.note }
    }
}

/// Nota excluída: fica lembrada para a exclusão chegar aos outros aparelhos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gone {
    pub id: Id,
    pub at: i64,
}

/// Uma entrada do feed de mudanças do aparelho.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Put(Note),
    Gone(Gone),
}

/// Um pedaço do feed: o que mudou depois de um cursor, até `seq`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Feed {
    pub seq: i64,
    pub more: bool,
    pub notes: Vec<Shot>,
    pub gone: Vec<Gone>,
}

/// O que um lote de mudanças fez no feed local (`from` → `to`) e as imagens que faltam.
/// Se `from` era o cursor de quem mandou, o que veio até `to` foi só o lote dele.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Took {
    pub from: i64,
    pub to: i64,
    pub need: Vec<Id>,
}

/// Outro aparelho com que este sincroniza, e até onde já foi em cada sentido.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub pull: i64,
    #[serde(default)]
    pub push: i64,
}

impl Link {
    pub fn new(url: impl Into<String>, token: impl Into<String>) -> Self {
        Self { url: url.into(), token: token.into(), pull: 0, push: 0 }
    }
}

/// Onde o aparelho guarda os links e o próprio id.
pub trait Links: Send + Sync {
    fn node(&self) -> Result<Id>;
    fn links(&self) -> Result<Vec<Link>>;
    /// Novo link, ou token novo para um que já existe (os cursores ficam).
    fn join(&self, l: &Link) -> Result<()>;
    fn part(&self, url: &str) -> Result<bool>;
    fn moved(&self, url: &str, pull: i64, push: i64) -> Result<()>;
}
