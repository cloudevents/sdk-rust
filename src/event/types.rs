use chrono::{DateTime, Utc};
use snafu;
use std::prelude::v1::*;
use url::Url;

/// Trait to define conversion to [`Url`]
pub trait TryIntoUrl {
    fn into_url(self) -> Result<Url, url::ParseError>;
}

impl TryIntoUrl for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }
}

impl TryIntoUrl for &str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)
    }
}

impl TryIntoUrl for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        self.as_str().into_url()
    }
}

/// Trait to define conversion to [`DateTime`]
pub trait TryIntoTime {
    fn into_time(self) -> Result<DateTime<Utc>, chrono::ParseError>;
}

impl TryIntoTime for DateTime<Utc> {
    fn into_time(self) -> Result<DateTime<Utc>, chrono::ParseError> {
        Ok(self)
    }
}

impl TryIntoTime for &str {
    fn into_time(self) -> Result<DateTime<Utc>, chrono::ParseError> {
        Ok(DateTime::<Utc>::from(DateTime::parse_from_rfc3339(self)?))
    }
}

impl TryIntoTime for String {
    fn into_time(self) -> Result<DateTime<Utc>, chrono::ParseError> {
        self.as_str().into_time()
    }
}
