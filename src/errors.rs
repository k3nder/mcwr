use mcd::errors::FetchError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("FETCH ERROR")]
    FetchError(#[from] FetchError),
    #[error("META ERROR")]
    InitMetaError(#[from] WritingError)
}

#[derive(Debug, Error)]
pub enum ReadingError {
    #[error("Error reading")]
    ReadError(#[from] std::io::Error),
    #[error("Error deserializing file")]
    DeserializeError(#[from] toml::de::Error)
}

#[derive(Debug, Error)]
pub enum WritingError {
    #[error("Error writing")]
    ReadError(#[from] std::io::Error),
    #[error("Error serializing file")]
    DeserializeError(#[from] toml::ser::Error)
}
