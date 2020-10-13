use snafu::Snafu;

/// Represents an error during serialization/deserialization process
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Wrong encoding"))]
    WrongEncoding {},
    #[snafu(display("{}", source))]
    #[snafu(context(false))]
    UnknownSpecVersion {
        source: crate::event::UnknownSpecVersion,
    },
    #[snafu(display("Unknown attribute in this spec version: {}", name))]
    UnknownAttribute { name: String },
    #[snafu(display("Error while building the final event: {}", source))]
    #[snafu(context(false))]
    EventBuilderError {
        source: crate::event::EventBuilderError,
    },
    #[snafu(display("Error while parsing a time string: {}", source))]
    #[snafu(context(false))]
    ParseTimeError { source: chrono::ParseError },
    #[snafu(display("Error while parsing a url: {}", source))]
    #[snafu(context(false))]
    ParseUrlError { source: url::ParseError },
    #[snafu(display("Error while decoding base64: {}", source))]
    #[snafu(context(false))]
    Base64DecodingError { source: base64::DecodeError },
    #[snafu(display("Error while serializing/deserializing to json: {}", source))]
    #[snafu(context(false))]
    SerdeJsonError { source: serde_json::Error },
    #[snafu(display("IO Error: {}", source))]
    #[snafu(context(false))]
    IOError { source: std::io::Error },
    #[snafu(display("Other error: {}", source))]
    Other {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Result type alias for return values during serialization/deserialization process
pub type Result<T> = std::result::Result<T, Error>;
