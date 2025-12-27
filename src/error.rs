use thiserror::Error;

/// dimicon library error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid image reference format: {0}")]
    InvalidImageReference(String),

    #[error("Rate limited by Docker Hub")]
    RateLimited,

    #[error("Image icon not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
