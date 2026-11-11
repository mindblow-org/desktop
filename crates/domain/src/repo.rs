use crate::{Id, Note, Result};

pub trait Repo: Send + Sync {
    fn all(&self) -> Result<Vec<Note>>;
    fn get(&self, id: Id) -> Result<Option<Note>>;
    fn put(&self, n: &Note) -> Result<()>;
    fn del(&self, id: Id) -> Result<bool>;
    fn head(&self) -> Result<Option<f64>>;
}
