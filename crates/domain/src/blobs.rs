use crate::{Id, Result};

pub trait Blobs: Send + Sync {
    fn put(&self, id: Id, mime: &str, data: &[u8]) -> Result<()>;
    fn get(&self, id: Id) -> Result<Option<(String, Vec<u8>)>>;
    fn del(&self, id: Id) -> Result<()>;
}
