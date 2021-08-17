mod deserializer;
mod headers;

use crate::{
    message::{Error, MessageDeserializer},
    Event,
};
use deserializer::Deserializer;
pub use headers::Headers;
mod serializer;

pub use serializer::Builder;
pub use serializer::Serializer;

pub static SPEC_VERSION_HEADER: &str = "ce-specversion";

/// Turn a pile of HTTP headers and a body into a CloudEvent
pub fn to_event<'a, T: Headers<'a>>(
    headers: &'a T,
    body: Vec<u8>,
) -> std::result::Result<Event, Error> {
    MessageDeserializer::into_event(Deserializer::new(headers, body))
}

pub fn header_prefix(name: &str) -> String {
    super::header_prefix("ce-", name)
}
