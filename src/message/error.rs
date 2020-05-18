use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Wrong encoding"))]
    WrongEncoding {},
    #[snafu(display("{}", source))]
    #[snafu(context(false))]
    InvalidSpecVersion {
        source: crate::event::spec_version::InvalidSpecVersion,
    },
    #[snafu(display("Unrecognized attribute name: {}", name))]
    UnrecognizedAttributeName { name: String },
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
    Other { source: Box<dyn std::error::Error> },
}

pub type SerializationResult = Result<(), Error>;
pub type DeserializationResult = Result<(), Error>;
