use crate::{Change, Id, Note, Result};

pub trait Repo: Send + Sync {
    fn all(&self) -> Result<Vec<Note>>;
    fn get(&self, id: Id) -> Result<Option<Note>>;
    /// Grava a nota e põe no fim do feed. Se ela tinha sido excluída, esquece a exclusão.
    fn put(&self, n: &Note) -> Result<()>;
    /// Exclui a nota e guarda a exclusão (em `at`) no feed. Diz se ela existia.
    fn del(&self, id: Id, at: i64) -> Result<bool>;
    /// Guarda a exclusão de uma nota que este aparelho nem tem, se for mais nova que a que já há.
    fn bury(&self, id: Id, at: i64) -> Result<()>;
    /// Quando a nota foi excluída, se foi.
    fn gone(&self, id: Id) -> Result<Option<i64>>;
    fn head(&self) -> Result<Option<f64>>;
    /// Posição atual do feed.
    fn seq(&self) -> Result<i64>;
    /// Até `limit` mudanças depois de `since`, em ordem.
    fn feed(&self, since: i64, limit: usize) -> Result<Vec<(i64, Change)>>;
}
