use thiserror::Error;

/// dimicon library error types
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("invalid image reference: {0}")]
    InvalidImageReference(&'static str),

    #[error("rate limited by upstream service")]
    RateLimited,
}

pub type Result<T> = std::result::Result<T, Error>;
