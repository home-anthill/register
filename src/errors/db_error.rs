#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("document already exists")]
    AlreadyExists,
    #[error("{0}")]
    Other(String),
}

impl DbError {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}
