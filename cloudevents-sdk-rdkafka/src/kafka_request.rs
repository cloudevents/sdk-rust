use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use rdkafka::producer::{FutureRecord};
use cloudevents::Event;
use rdkafka::message::{OwnedHeaders, ToBytes};

/// Wrapper for [`RequestBuilder`] that implements [`StructuredSerializer`] & [`BinarySerializer`] traits
pub struct RequestSerializer<'a,K: ToBytes + ?Sized,P: ToBytes + ?Sized> {
    req: FutureRecord<'a,K,P>,
    headers: OwnedHeaders,
}


impl<'a,K: ToBytes + ?Sized> RequestSerializer<'a,K,Vec<u8>> {
    pub fn new(FutureRec: FutureRecord<'a,K,Vec<u8>>) -> RequestSerializer<'a,K,Vec<u8>> {
        
        RequestSerializer { req: FutureRec, headers: OwnedHeaders::new()}

    }
}

impl<'a,K: ToBytes + ?Sized> BinarySerializer<FutureRecord<'a,K,Vec<u8>>> for RequestSerializer<'a,K,Vec<u8>> {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.headers = self.headers.add("ce_specversion", spec_version.as_str());

        self.req = self
            .req
            .headers(self.headers.clone());
        Ok(
            self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.headers = self.headers.add(
            &headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone()[..],
            &value.to_string()[..],
        );
        
        self.req = self.req.headers(self.headers.clone());
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.headers = self.headers.add(&attribute_name_to_header!(name)[..], &value.to_string()[..]);

        self.req = self
            .req
            .headers(self.headers.clone());
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<FutureRecord<'a,K,Vec<u8>>> {
        
        Ok(self.req.payload(&bytes))
    }

    fn end(self) -> Result<FutureRecord<'a,K,Vec<u8>>> {
        Ok(self.req)
    }
}

impl<'a,K: ToBytes + ?Sized> StructuredSerializer<FutureRecord<'a,K,Vec<u8>>> for RequestSerializer<'a,K,Vec<u8>> {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<FutureRecord<'a,K,Vec<u8>>> {
        self.headers = self.headers.add("content-type","application/cloudevents+json",);
        
        Ok(self
            .req
            .payload(&bytes)
            .headers(
                self.headers.clone()
            )
        )
    }
}

/// Method to fill a [`RequestBuilder`] with an [`Event`]
pub fn event_to_request<'a,K: ToBytes + ?Sized>(event: Event, request_builder: FutureRecord<'a,K,Vec<u8>>) -> Result<FutureRecord<'a,K,Vec<u8>>> {
    BinaryDeserializer::deserialize_binary(event, RequestSerializer::new(request_builder))
}

/// Extention Trait for [`RequestBuilder`] which acts as a wrapper for the function [`event_to_request()`]
pub trait FutureRecordExt<'a,K: ToBytes + ?Sized> {
    fn event(self, event: Event) -> Result<FutureRecord<'a,K,Vec<u8>>>;
}

impl<'a,K: ToBytes + ?Sized> FutureRecordExt<'a,K> for FutureRecord<'a,K,Vec<u8>> {
    fn event(self, event: Event) -> Result<FutureRecord<'a,K,Vec<u8>>> {
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
