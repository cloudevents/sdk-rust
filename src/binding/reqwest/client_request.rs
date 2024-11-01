use reqwest_lib as reqwest;

use crate::binding::{
    http::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_BATCH_JSON_HEADER, CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use crate::Event;
use reqwest::RequestBuilder;

// TODO: Ideally, we'd only need to implement binding::http::Builder
// for reqwest::RequestBuilder here, but because the latter is a
// consuming builder, we'd need an intermediate struct similar to
// warp's to adapt that interface. Unfortunately, the reqwest builder
// doesn't implement the Default trait, so I can't use take() as
// warp's Adapter does, and I've yet to come up with another
// solution. So for now, we continue to implement BinarySerializer
// directly in here.

/// Wrapper for [`RequestBuilder`] that implements [`StructuredSerializer`] & [`BinarySerializer`] traits.
pub struct RequestSerializer {
    req: RequestBuilder,
}

impl RequestSerializer {
    pub fn new(req: RequestBuilder) -> RequestSerializer {
        RequestSerializer { req }
    }
}

impl BinarySerializer<RequestBuilder> for RequestSerializer {
    fn set_spec_version(mut self, spec_ver: SpecVersion) -> Result<Self> {
        self.req = self.req.header(SPEC_VERSION_HEADER, spec_ver.to_string());
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        let key = &header_prefix(name);
        self.req = self.req.header(key, value.to_string());
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        let key = &header_prefix(name);
        self.req = self.req.header(key, value.to_string());
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<RequestBuilder> {
        Ok(self.req.body(bytes))
    }

    fn end(self) -> Result<RequestBuilder> {
        Ok(self.req)
    }
}

impl StructuredSerializer<RequestBuilder> for RequestSerializer {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<RequestBuilder> {
        Ok(self
            .req
            .header(reqwest::header::CONTENT_TYPE, CLOUDEVENTS_JSON_HEADER)
            .body(bytes))
    }
}

/// Method to fill a [`RequestBuilder`] with an [`Event`].
pub fn event_to_request(event: Event, request_builder: RequestBuilder) -> Result<RequestBuilder> {
    BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(request_builder))
}

/// Method to fill a [`RequestBuilder`] with a batched [`Vec<Event>`].
pub fn events_to_request(
    events: Vec<Event>,
    request_builder: RequestBuilder,
) -> Result<RequestBuilder> {
    let bytes = serde_json::to_vec(&events)?;
    Ok(request_builder
        .header(reqwest::header::CONTENT_TYPE, CLOUDEVENTS_BATCH_JSON_HEADER)
        .body(bytes))
}

/// Extension Trait for [`RequestBuilder`] which acts as a wrapper for the function [`event_to_request()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait RequestBuilderExt: private::Sealed {
    /// Write in this [`RequestBuilder`] the provided [`Event`]. Similar to invoking [`Event`].
    fn event(self, event: Event) -> Result<RequestBuilder>;
    /// Write in this [`RequestBuilder`] the provided batched [`Vec<Event>`].
    fn events(self, events: Vec<Event>) -> Result<RequestBuilder>;
}

impl RequestBuilderExt for RequestBuilder {
    fn event(self, event: Event) -> Result<RequestBuilder> {
        event_to_request(event, self)
    }

    fn events(self, events: Vec<Event>) -> Result<RequestBuilder> {
        events_to_request(events, self)
    }
}

// Sealing the RequestBuilderExt
mod private {
    use reqwest_lib as reqwest;

    pub trait Sealed {}
    impl Sealed for reqwest::RequestBuilder {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use reqwest_lib as reqwest;

    use crate::message::StructuredDeserializer;
    use crate::test::fixtures;

    #[tokio::test]
    async fn test_request() {
        let url = mockito::server_url();
        let m = mockito::mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .match_header("ce-type", "test_event.test_application")
            .match_header("ce-source", "http://localhost/")
            .match_header("ce-someint", "10")
            .match_body(Matcher::Missing)
            .create();

        let input = fixtures::v10::minimal_string_extension();

        let client = reqwest::Client::new();
        client
            .post(&url)
            .event(input)
            .unwrap()
            .send()
            .await
            .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_request_with_full_data() {
        let url = mockito::server_url();
        let m = mockito::mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .with_header("ce-type", "test_event.test_application")
            .with_header("ce-source", "http://localhost/")
            .with_header("ce-subject", "cloudevents-sdk")
            .with_header("content-type", "application/json")
            .with_header("ce-string_ex", "val")
            .with_header("ce-int_ex", "10")
            .with_header("ce-bool_ex", "true")
            .with_header("ce-time", &fixtures::time().to_rfc3339())
            .match_body(Matcher::Exact(fixtures::json_data().to_string()))
            .create();

        let input = fixtures::v10::full_binary_json_data_string_extension();

        let client = reqwest::Client::new();

        client
            .post(&url)
            .event(input)
            .unwrap()
            .send()
            .await
            .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_structured_request_with_full_data() {
        let input = fixtures::v10::full_json_data_string_extension();

        let url = mockito::server_url();
        let m = mockito::mock("POST", "/")
            .match_header("content-type", "application/cloudevents+json")
            .match_body(Matcher::Exact(serde_json::to_string(&input).unwrap()))
            .create();

        let client = reqwest::Client::new();
        StructuredDeserializer::deserialize_structured(
            input,
            RequestSerializer::new(client.post(&url)),
        )
        .unwrap()
        .send()
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_batched_request() {
        let input = vec![fixtures::v10::full_json_data_string_extension()];

        let url = mockito::server_url();
        let m = mockito::mock("POST", "/")
            .match_header("content-type", "application/cloudevents-batch+json")
            .match_body(Matcher::Exact(serde_json::to_string(&input).unwrap()))
            .create();

        let client = reqwest::Client::new();
        client
            .post(&url)
            .events(input)
            .unwrap()
            .send()
            .await
            .unwrap();

        m.assert();
    }
}
