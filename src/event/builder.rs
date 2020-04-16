use super::EventBuilderV10;

/// Builder to create [`Event`]:
/// ```
/// use cloudevents::EventBuilder;
/// use chrono::Utc;
/// use url::Url;
///
/// let event = EventBuilder::v10()
///     .id("my_event.my_application")
///     .source(Url::parse("http://localhost:8080").unwrap())
///     .time(Utc::now())
///     .build();
/// ```
pub struct EventBuilder {}

impl EventBuilder {
    /// Creates a new builder for CloudEvents V1.0
    pub fn v10() -> EventBuilderV10 {
        return EventBuilderV10::new();
    }
}
