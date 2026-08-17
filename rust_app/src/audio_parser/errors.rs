use thiserror::Error;
#[derive(Debug, Error)]
pub enum AudioParserError {
    #[error("{0}")]
    InputAudioFileError(String),
    #[error("{0}")]
    InvalidFileHeaderError(String),
}
