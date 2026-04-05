#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
