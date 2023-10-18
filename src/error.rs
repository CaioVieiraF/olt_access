#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error(transparent)]
    Connection(#[from] ssh2::Error),
}
