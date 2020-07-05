use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use rdkafka::producer::{FutureRecord};
use cloudevents::Event;
//use bytes::Bytes;
use rdkafka::message::{OwnedHeaders, ToBytes};

/// Wrapper for [`RequestBuilder`] that implements [`StructuredSerializer`] & [`BinarySerializer`] traits
pub struct RequestSerializer<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> {
    req: FutureRecord<'a,K,P>,
    headers: OwnedHeaders,
}

impl<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> RequestSerializer<'a,K,P> {
    pub fn new(FutureRec: FutureRecord<'a,K,P>) -> RequestSerializer<'a,K,P> {
        let req = FutureRec;
        let headers = OwnedHeaders::new();
        RequestSerializer { req, headers }
    }
}

impl<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> BinarySerializer<FutureRecord<'a,K,P>> for RequestSerializer<'a,K,P> {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.req = self
            .req
            .headers(self.headers.add("ce_specversion", spec_version.as_str()));
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.req = self.req.headers(self.headers.add(
            &headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone()[..],
            &value.to_string()[..],
        ));
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.req = self
            .req
            .headers(self.headers.add(&attribute_name_to_header!(name)[..], &value.to_string()[..]));
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<FutureRecord<'a,K,P>> {
        Ok(self.req.payload(bytes.to_bytes()))
    }

    fn end(self) -> Result<FutureRecord<'a,K,P>> {
        Ok(self.req)
    }
}

impl<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> StructuredSerializer<FutureRecord<'a,K,P>> for RequestSerializer<'a,K,P> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<FutureRecord<'a,K,P>> {
        Ok(self
            .req
            .payload(bytes.to_bytes())
            .headers(
                self.headers.add("content-type",
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
                )
            )
        )
    }
}

/// Method to fill a [`RequestBuilder`] with an [`Event`]
pub fn event_to_request<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized>(event: Event, request_builder: FutureRecord<'a,K,P>) -> Result<FutureRecord<'a,K,P>> {
    BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(request_builder))
}

/// Extention Trait for [`RequestBuilder`] which acts as a wrapper for the function [`event_to_request()`]
pub trait FutureRecordExt<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> {
    fn event(self, event: Event) -> Result<FutureRecord<'a,K,P>>;
}

impl<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> FutureRecordExt<'a,K,P> for FutureRecord<'a,K,P> {
    fn event(self, event: Event) -> Result<FutureRecord<'a,K,P>> {
        event_to_request(event, self)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    use cloudevents::message::StructuredDeserializer;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;
    use url::Url;

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
            .source(Url::from_str("http://localhost/").unwrap())
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
            .source(Url::from_str("http://localhost").unwrap())
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
            .source(Url::from_str("http://localhost").unwrap())
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
}*/
