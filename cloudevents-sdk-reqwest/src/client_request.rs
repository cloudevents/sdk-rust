use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use reqwest::RequestBuilder;
use std::str::FromStr;

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
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.req = self
            .req
            .header(headers::SPEC_VERSION_HEADER.clone(), spec_version.as_str());
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.req = self.req.header(
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            value.to_string(),
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.req = self
            .req
            .header(attribute_name_to_header!(name)?, value.to_string());
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
            .header(
                reqwest::header::CONTENT_TYPE,
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
            )
            .body(bytes))
    }
}

/// Method to fill a [`RequestBuilder`] with an [`Event`].
pub fn event_to_request(event: Event, request_builder: RequestBuilder) -> Result<RequestBuilder> {
    BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(request_builder))
}

/// Extension Trait for [`RequestBuilder`] which acts as a wrapper for the function [`event_to_request()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait RequestBuilderExt: private::Sealed {
    /// Write in this [`RequestBuilder`] the provided [`Event`]. Similar to invoking [`Event`].
    fn event(self, event: Event) -> Result<RequestBuilder>;
}

impl RequestBuilderExt for RequestBuilder {
    fn event(self, event: Event) -> Result<RequestBuilder> {
        event_to_request(event, self)
    }
}

// Sealing the RequestBuilderExt
mod private {
    pub trait Sealed {}
    impl Sealed for reqwest::RequestBuilder {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    use cloudevents::message::StructuredDeserializer;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;

    #[tokio::test]
    async fn test_request() {
        let url = mockito::server_url();
        let m = mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .match_header("ce-type", "example.test")
            .match_header("ce-source", "http://localhost/")
            .match_header("ce-someint", "10")
            .match_body(Matcher::Missing)
            .create();

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .extension("someint", "10")
            .build()
            .unwrap();

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
        let j = json!({"hello": "world"});

        let url = mockito::server_url();
        let m = mock("POST", "/")
            .match_header("ce-specversion", "1.0")
            .match_header("ce-id", "0001")
            .match_header("ce-type", "example.test")
            .match_header("ce-source", "http://localhost/")
            .match_header("content-type", "application/json")
            .match_header("ce-someint", "10")
            .match_body(Matcher::Exact(j.to_string()))
            .create();

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

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
        let j = json!({"hello": "world"});

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let url = mockito::server_url();
        let m = mock("POST", "/")
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
}
