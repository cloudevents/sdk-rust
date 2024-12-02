pub mod builder;
pub mod deserializer;
mod headers;

use crate::{
    message::{Error, MessageDeserializer},
    Event,
};
use deserializer::Deserializer;
pub use headers::Headers;
mod serializer;

pub use builder::Builder;
use core::convert::TryFrom;
use http::Response;
use http_0_2 as http;
pub use serializer::Serializer;
use std::convert::TryInto;
use std::fmt::Debug;

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

impl<T> TryFrom<Response<T>> for Event
where
    T: TryInto<Vec<u8>>,
    <T as TryInto<Vec<u8>>>::Error: Debug,
{
    type Error = crate::message::Error;

    fn try_from(response: Response<T>) -> Result<Self, Self::Error> {
        let headers = response.headers().to_owned();
        let body = T::try_into(response.into_body()).unwrap();

        to_event(&headers, body)
    }
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;
    use crate::Event;
    use core::convert::TryFrom;
    use http::Response;
    use http_0_2 as http;

    #[test]
    fn test_response_to_event() {
        let event = fixtures::v10::minimal_string_extension();

        let response = Response::builder()
            .header("ce-id", fixtures::id())
            .header("ce-source", fixtures::source())
            .header("ce-type", fixtures::ty())
            .header("ce-specversion", "1.0")
            .header("ce-someint", "10")
            .body(Vec::new())
            .unwrap();

        assert_eq!(event, Event::try_from(response).unwrap());
    }
}
