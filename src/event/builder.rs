use super::Event;
use snafu::Snafu;

/// Trait to implement a builder for [`Event`]:
/// ```
/// use cloudevents::event::{EventBuilderV10, EventBuilder};
/// use chrono::Utc;
/// use url::Url;
///
/// let event = EventBuilderV10::new()
///     .id("my_event.my_application")
///     .source("http://localhost:8080")
///     .ty("example.demo")
///     .time(Utc::now())
///     .build()
///     .unwrap();
/// ```
///
/// You can create an [`EventBuilder`] starting from an existing [`Event`] using the [`From`] trait.
/// You can create a default [`EventBuilder`] setting default values for some attributes.
pub trait EventBuilder
where
    Self: Clone + Sized + From<Event> + Default,
{
    /// Create a new empty builder
    fn new() -> Self;

    /// Build [`Event`]
    fn build(self) -> Result<Event, Error>;
}

/// Represents an error during build process
#[derive(Debug, Snafu, Clone)]
pub enum Error {
    #[snafu(display("Missing required attribute {}", attribute_name))]
    MissingRequiredAttribute { attribute_name: &'static str },
    #[snafu(display(
        "Error while setting attribute '{}' with timestamp type: {}",
        attribute_name,
        source
    ))]
    ParseTimeError {
        attribute_name: &'static str,
        source: chrono::ParseError,
    },
    #[snafu(display(
        "Error while setting attribute '{}' with uri/uriref type: {}",
        attribute_name,
        source
    ))]
    ParseUrlError {
        attribute_name: &'static str,
        source: url::ParseError,
    },
}
