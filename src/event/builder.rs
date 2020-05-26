use super::Event;
use snafu::Snafu;

/// Builder to create [`super::Event`]:
/// ```
/// use cloudevents::event::{EventBuilderV10, EventBuilder};
/// use chrono::Utc;
/// use url::Url;
///
/// let event = EventBuilderV10::new()
///     .id("my_event.my_application")
///     .source("http://localhost:8080")
///     .time(Utc::now())
///     .build()?;
/// ```
pub trait EventBuilder where Self: Clone + Sized {
    /// Create a new builder copying the contents of the provided [`Event`].
    fn from(event: Event) -> Self;

    /// Create a new empty builder
    fn new() -> Self;

    /// Build [`super::Event`]
    fn build(self) -> Result<super::Event, Error>;
}

/// Represents an error during build process
#[derive(Debug, Snafu, Clone)]
pub enum Error {
    #[snafu(display("Missing required attribute {}", attribute_name))]
    MissingRequiredAttribute{ attribute_name: &'static str },
    #[snafu(display("Error while setting attribute '{}' with timestamp type: {}", attribute_name, source))]
    ParseTimeError { attribute_name: &'static str, source: chrono::ParseError },
    #[snafu(display("Error while setting attribute '{}' with uri/uriref type: {}", attribute_name, source))]
    ParseUrlError { attribute_name: &'static str, source: url::ParseError },
}

impl <T: EventBuilder> Default for T {
    fn default() -> T {
        T::from(Event::default())
    }
}