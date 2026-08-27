use thiserror::Error;
#[derive(Clone, Debug, Error)]
pub enum AudioParserError {
    #[error("{0}")]
    InputAudioFileError(String),
    #[error("{0}")]
    InvalidFileHeaderError(String),
    #[error("invalid byte count: expected {expected}, got {actual}")]
    InvalidByteCount { expected: usize, actual: usize },
}

#[derive(Clone, Debug, Error)]
pub enum RenderingError {
    #[error("{0}")]
    ChartSetupError(String),
    #[error("{0}")]
    ChartDrawError(String),
}
