//! Provides facilities to implement Protocol Bindings.
//!
//! Note: these APIs should be considered unstable and subject to changes.

mod deserializer;
mod encoding;
mod error;
mod serializer;
mod types;

pub use deserializer::*;
pub use encoding::*;
pub use error::*;
pub use serializer::*;
pub use types::MessageAttributeValue;

/// Returns true if `content_type` has media type `application/cloudevents+json`
/// (case-insensitively, ignoring optional media type parameters per RFC 9110 Section 8.3.1).
#[cfg(any(
    feature = "http-binding",
    feature = "http-0-2-binding",
    feature = "actix",
    feature = "reqwest",
    feature = "warp",
    feature = "axum",
    feature = "poem",
    feature = "rdkafka",
    feature = "fe2o3-amqp"
))]
pub(crate) fn is_cloudevents_json_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .eq_ignore_ascii_case(crate::binding::CLOUDEVENTS_JSON_HEADER)
}
