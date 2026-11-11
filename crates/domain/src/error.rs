#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("nota não encontrada")]
    Missing,
    #[error("item não encontrado")]
    NoItem,
    #[error("imagem não encontrada")]
    NoImage,
    #[error("{0}")]
    Invalid(&'static str),
    #[error("falha no armazenamento: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, Error>;
