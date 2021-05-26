use super::headers;
use async_trait::async_trait;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use std::str::FromStr;
use surf::{Error, Request};

/// Wrapper for [`Request`] that implements [`StructuredSerializer`] and [`BinarySerializer`].
pub struct RequestSerializer {
    builder: Request,
}

impl RequestSerializer {
    pub fn new(builder: Request) -> RequestSerializer {
        RequestSerializer { builder }
    }
}

impl BinarySerializer<Request> for RequestSerializer {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.builder
            .insert_header(headers::SPEC_VERSION_HEADER.clone(), spec_version.as_str());
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.insert_header(
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            value.to_string().as_str(),
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder
            .insert_header(attribute_name_to_header!(name)?, value.to_string().as_str());
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<Request> {
        self.builder.set_body(bytes);
        Ok(self.builder)
    }

    fn end(self) -> Result<Request> {
        Ok(self.builder)
    }
}

impl StructuredSerializer<Request> for RequestSerializer {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<Request> {
        self.builder.insert_header(
            surf::http::headers::CONTENT_TYPE,
            headers::CLOUDEVENTS_JSON_HEADER.clone(),
        );
        self.builder.set_body(bytes);
        Ok(self.builder)
    }
}

/// Method to fill an [`Request`] with an [`Event`].
pub async fn event_to_request(
    event: Event,
    request: Request,
) -> std::result::Result<Request, surf::Error> {
    BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(request))
        .map_err(|e| Error::new(400, e))
}

/// Extension Trait for [`Request`] which acts as a wrapper for the function [`event_to_Request()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[async_trait]
pub trait RequestExt: private::Sealed {
    /// Fill this [`Request`] with an [`Event`].
    async fn event(self, event: Event) -> std::result::Result<Request, surf::Error>;
}

#[async_trait]
impl RequestExt for Request {
    async fn event(self, event: Event) -> std::result::Result<Request, surf::Error> {
        event_to_request(event, self).await
    }
}


// Sealing the RequestExt
mod private {
    pub trait Sealed {}
    impl Sealed for surf::Request {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudevents::message::StructuredDeserializer;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use mockito::{mock, Matcher};
    use serde_json::json;
    use surf::http;
    use surf::Url;

    #[async_std::test]
    async fn test_request() {
        let url = mockito::server_url();
        let m = mock("POST", "/")
            .with_header("content-type", "application/octet-stream")
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

        let req = Request::new(http::Method::Post, Url::parse(url.as_str()).unwrap());
        let evt = req.event(input).await.unwrap();
        let client = surf::Client::new();
        client.send(evt).await.unwrap();

        m.assert();
    }

    #[async_std::test]
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

        let req = Request::new(http::Method::Post, Url::parse(url.as_str()).unwrap());
        let evt = req.event(input).await.unwrap();
        let client = surf::Client::new();
        client.send(evt).await.unwrap();

        m.assert();
    }

    #[async_std::test]
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

        let req = Request::new(http::Method::Post, Url::parse(url.as_str()).unwrap());
        let client = surf::Client::new();
        let evt =
            StructuredDeserializer::deserialize_structured(input, RequestSerializer::new(req));
        client.send(evt.unwrap()).await.unwrap();

        m.assert();
    }
}
