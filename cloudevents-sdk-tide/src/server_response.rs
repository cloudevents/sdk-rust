use super::headers;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use std::str::FromStr;
use tide::{Error, Response};

/// Wrapper for [`Response`] that implements [`StructuredSerializer`] and [`BinarySerializer`].
pub struct ResponseSerializer {
    builder: Response,
}

impl ResponseSerializer {
    pub fn new(builder: Response) -> ResponseSerializer {
        ResponseSerializer { builder }
    }
}

impl BinarySerializer<Response> for ResponseSerializer {
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

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<Response> {
        self.builder.set_body(bytes);
        Ok(self.builder)
    }

    fn end(self) -> Result<Response> {
        Ok(self.builder)
    }
}

impl StructuredSerializer<Response> for ResponseSerializer {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<Response> {
        self.builder.insert_header(
            http_types::headers::CONTENT_TYPE,
            headers::CLOUDEVENTS_JSON_HEADER.clone(),
        );
        self.builder.set_body(bytes);
        Ok(self.builder)
    }
}

/// Method to fill an [`Response`] with an [`Event`].
pub fn event_to_response(
    event: Event,
    response: Response,
) -> std::result::Result<Response, tide::Error> {
    BinaryDeserializer::deserialize_binary(event, ResponseSerializer::new(response))
        .map_err(|e| Error::new(400, e))
}

/// Extension Trait for [`Response`] which acts as a wrapper for the function [`event_to_response()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait ResponseBuilderExt: private::Sealed {
    /// Fill this [`Response`] with an [`Event`].
    fn event(self, event: Event) -> std::result::Result<Response, tide::Error>;
}

impl ResponseBuilderExt for Response {
    fn event(self, event: Event) -> std::result::Result<Response, tide::Error> {
        event_to_response(event, self)
    }
}

// Sealing the ResponseBuilderExt
mod private {
    pub trait Sealed {}
    impl Sealed for tide::Response {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;
    use tide::Response;
    use tide_testing::TideTestingExt;
    #[async_std::test]
    async fn test_response() {
        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = Response::new(200).event(input).unwrap();

        assert_eq!(resp.header("ce-specversion").unwrap().as_str(), "1.0");
        assert_eq!(resp.header("ce-id").unwrap().as_str(), "0001");
        assert_eq!(resp.header("ce-type").unwrap().as_str(), "example.test");
        assert_eq!(
            resp.header("ce-source").unwrap().as_str(),
            "http://localhost/"
        );
        assert_eq!(resp.header("ce-someint").unwrap().as_str(), "10");
    }

    #[async_std::test]
    async fn test_response_with_full_data() {
        let j = json!({"hello": "world"});

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = Response::new(200).event(input).unwrap();

        assert_eq!(resp.header("ce-specversion").unwrap().as_str(), "1.0");
        assert_eq!(resp.header("ce-id").unwrap().as_str(), "0001");
        assert_eq!(resp.header("ce-type").unwrap().as_str(), "example.test");
        assert_eq!(
            resp.header("ce-source").unwrap().as_str(),
            "http://localhost/"
        );
        assert_eq!(
            resp.header("content-type").unwrap().as_str(),
            "application/json"
        );
        assert_eq!(resp.header("ce-someint").unwrap().as_str(), "10");
    }

    #[async_std::test]
    async fn test_response_in_service() {
        let mut app = tide::new();
        app.at("/").get(|_| async move {
            let j = json!({"hello":"world"});
            let input = EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                .data("application/json", j.clone())
                .extension("someint", "10")
                .build()
                .unwrap();

            let resp = Response::new(200).event(input).unwrap();
            Ok(resp)
        });

        match app.get("/").recv_string().await {
            Ok(r) => {
                println!("test_response_in_service:{}", r);
                r
            }
            Err(e) => panic!("Get String Failed {:?}", e),
        };
    }

    // use async_std::stream::Stream;
    // use bytes::{Bytes, BytesMut};
    // pub async fn load_stream<S>(mut stream: S) -> Result<Bytes, Error>
    // where
    //     S: Stream<Item = Result<Bytes, Error>> + Unpin,
    // {
    //     let mut data = BytesMut::new();
    //     while let Some(item) = stream.next().await {
    //         data.extend_from_slice(&item?);
    //     }
    //     Ok(data.freeze())
    // }
}
