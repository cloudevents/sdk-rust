use core::fmt::{self, Debug, Display};
use snafu::Snafu;
use std::prelude::v1::*;

#[cfg(feature = "std")]
use url;
#[cfg(not(feature = "std"))]
use String as url;

pub struct DisplayError<T>(pub T);

impl<T> Debug for DisplayError<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Display for DisplayError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> snafu::Error for DisplayError<T> where T: Display + Debug {}

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
    ParseTimeError {
        #[snafu(source(from(chrono::ParseError, DisplayError)))]
        source: DisplayError<chrono::ParseError>,
    },

    #[snafu(display("Error while parsing a url: {}", source))]
    #[snafu(context(false))]
    ParseUrlError {
        #[cfg(not(feature = "std"))]
        #[snafu(source(from(String, DisplayError)))]
        source: DisplayError<String>,

        #[cfg(feature = "std")]
        #[snafu(source(from(url::ParseError, DisplayError)))]
        source: DisplayError<url::ParseError>,
    },

    #[snafu(display("Error while decoding base64: {}", source))]
    #[snafu(context(false))]
    Base64DecodingError {
        #[snafu(source(from(base64::DecodeError, DisplayError)))]
        source: DisplayError<base64::DecodeError>,
    },
    #[snafu(display("Error while serializing/deserializing to json: {}", source))]
    #[snafu(context(false))]
    SerdeJsonError {
        #[snafu(source(from(serde_json::Error, DisplayError)))]
        source: DisplayError<serde_json::Error>,
    },

    #[cfg(feature = "std")]
    #[snafu(display("IO Error: {}", source))]
    #[snafu(context(false))]
    IOError { source: std::io::Error },

    #[cfg(feature = "std")]
    #[snafu(display("Other error: {}", source))]
    Other {
        //#[snafu(source(from(Box<dyn core_error::Error>, DisplayError)))]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Result type alias for return values during serialization/deserialization process
pub type Result<T> = std::result::Result<T, Error>;
