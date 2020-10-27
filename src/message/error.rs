use thiserror::Error;

/// Represents an error during serialization/deserialization process
#[derive(Debug, Error)]
pub enum Error {
    #[error("Wrong encoding")]
    WrongEncoding {},
    #[error(transparent)]
    UnknownSpecVersion {
        #[from]
        source: crate::event::UnknownSpecVersion,
    },
    #[error("Unknown attribute in this spec version: {name}")]
    UnknownAttribute { name: String },
    #[error("Error while building the final event: {source}")]
    EventBuilderError {
        #[from]
        source: crate::event::EventBuilderError,
    },
    #[error("Error while parsing a time string: {source}")]
    ParseTimeError {
        #[from]
        source: chrono::ParseError,
    },
    #[error("Error while parsing a url: {source}")]
    ParseUrlError {
        #[from]
        source: url::ParseError,
    },
    #[error("Error while decoding base64: {source}")]
    Base64DecodingError {
        #[from]
        source: base64::DecodeError,
    },
    #[error("Error while serializing/deserializing to json: {source}")]
    SerdeJsonError {
        #[from]
        source: serde_json::Error,
    },
    #[error("IO Error: {source}")]
    IOError {
        #[from]
        source: std::io::Error,
    },
    #[error("Other error: {}", source)]
    Other {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Result type alias for return values during serialization/deserialization process
pub type Result<T> = std::result::Result<T, Error>;
