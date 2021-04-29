pub use self::uri::ParseUriError;
pub use self::uri::TryIntoUri;
pub use self::uri::Uri;

pub use urireference::UriReference;

pub use chrono::{DateTime, Utc};
pub use time::TryIntoTime;

mod uri {
    #[cfg(feature = "std")]
    pub use url::Url as Uri;
    #[cfg(not(feature = "std"))]
    pub use String as Uri;

    #[cfg(feature = "std")]
    pub use url::ParseError as ParseUriError;
    #[cfg(not(feature = "std"))]
    pub use None as ParseUriError;

    /// Trait to define conversion to [`Uri`]
    pub trait TryIntoUri {
        fn into_uri(self) -> Result<Uri, ParseUriError>;
    }

    #[cfg(feature = "std")]
    impl TryIntoUri for Uri {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            Ok(self)
        }
    }

    #[cfg(feature = "std")]
    impl TryIntoUri for &str {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            url::Url::parse(self)
        }
    }

    #[cfg(feature = "std")]
    impl TryIntoUri for String {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            self.as_str().into_uri()
        }
    }

    #[cfg(not(feature = "std"))]
    impl TryIntoUri for Uri {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            Ok(self)
        }
    }

    #[cfg(not(feature = "std"))]
    impl TryIntoUri for &str {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            Ok(String::from(self))
        }
    }

    #[cfg(not(feature = "std"))]
    impl TryIntoUri for String {
        fn into_uri(self) -> Result<Uri, ParseUriError> {
            Ok(self)
        }
    }
}

mod urireference {
    /// The URI-reference type.
    ///
    /// The URI reference can be a URI, or just a relative path.
    ///
    /// As the [`types::Url`] type can only represent an absolute URL, we are falling back to a string
    /// here.
    ///
    /// Also see:
    /// * <https://github.com/cloudevents/spec/blob/v1.0.1/spec.md#type-system>
    /// * <https://tools.ietf.org/html/rfc3986#section-4.1>
    pub type UriReference = String;
}

mod time {
    use chrono::{DateTime, Utc};

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
}
