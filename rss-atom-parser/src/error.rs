#[derive(Debug, thiserror::Error)]
#[error("Error near tag `{tag}`, {message}")]
pub struct ParserError {
    pub tag: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct FetchError {
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct FeedNotFoundError {
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct SerializationError {
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error(transparent)]
    Parse(#[from] ParserError),
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    NotFound(#[from] FeedNotFoundError),
    #[error(transparent)]
    Serialization(#[from] SerializationError),
}
