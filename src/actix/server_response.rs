use super::headers;
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use crate::Event;
use actix_web::http::{HeaderName, HeaderValue, StatusCode};
use actix_web::HttpResponse;
use actix_web::HttpResponseBuilder;
use std::str::FromStr;

/// Wrapper for [`HttpResponseBuilder`] that implements [`StructuredSerializer`] and [`BinarySerializer`].
pub struct HttpResponseSerializer {
    builder: HttpResponseBuilder,
}

impl HttpResponseSerializer {
    pub fn new(builder: HttpResponseBuilder) -> HttpResponseSerializer {
        HttpResponseSerializer { builder }
    }
}

impl BinarySerializer<HttpResponse> for HttpResponseSerializer {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.builder.insert_header((
            headers::SPEC_VERSION_HEADER,
            str_to_header_value!(spec_version.as_str())?,
        ));
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.insert_header((
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            str_to_header_value!(value.to_string().as_str())?,
        ));
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.insert_header((
            attribute_name_to_header!(name)?,
            str_to_header_value!(value.to_string().as_str())?,
        ));
        Ok(self)
    }

    fn end_with_data(mut self, bytes: Vec<u8>) -> Result<HttpResponse> {
        Ok(self.builder.body(bytes))
    }

    fn end(mut self) -> Result<HttpResponse> {
        Ok(self.builder.finish())
    }
}

impl StructuredSerializer<HttpResponse> for HttpResponseSerializer {
    fn set_structured_event(mut self, bytes: Vec<u8>) -> Result<HttpResponse> {
        Ok(self
            .builder
            .insert_header((
                actix_web::http::header::CONTENT_TYPE,
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
            ))
            .body(bytes))
    }
}

/// Method to fill an [`HttpResponseBuilder`] with an [`Event`].
pub fn event_to_response(
    event: Event,
    response: HttpResponseBuilder,
) -> std::result::Result<HttpResponse, actix_web::error::Error> {
    BinaryDeserializer::deserialize_binary(event, HttpResponseSerializer::new(response))
        .map_err(actix_web::error::ErrorBadRequest)
}

/// So that an actix-web handler may return an Event
impl actix_web::Responder for Event {
    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::build(StatusCode::OK).event(self).unwrap()
    }
}

/// Extension Trait for [`HttpResponseBuilder`] which acts as a wrapper for the function [`event_to_response()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait HttpResponseBuilderExt: private::Sealed {
    /// Fill this [`HttpResponseBuilder`] with an [`Event`].
    fn event(self, event: Event) -> std::result::Result<HttpResponse, actix_web::error::Error>;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn event(self, event: Event) -> std::result::Result<HttpResponse, actix_web::error::Error> {
        event_to_response(event, self)
    }
}

// Sealing the HttpResponseBuilderExt
mod private {
    pub trait Sealed {}
    impl Sealed for actix_web::HttpResponseBuilder {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{EventBuilder, EventBuilderV10};
    use actix_web::{http::StatusCode, test};
    use serde_json::json;

    #[actix_rt::test]
    async fn test_response() {
        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = HttpResponseBuilder::new(StatusCode::OK)
            .event(input)
            .unwrap();

        assert_eq!(
            resp.headers()
                .get("ce-specversion")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            "0001"
        );
        assert_eq!(
            resp.headers().get("ce-type").unwrap().to_str().unwrap(),
            "example.test"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost/"
        );
        assert_eq!(
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );
    }

    #[actix_rt::test]
    async fn test_response_with_full_data() {
        let j = json!({"hello": "world"});

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = HttpResponseBuilder::new(StatusCode::OK)
            .event(input)
            .unwrap();

        assert_eq!(
            resp.headers()
                .get("ce-specversion")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            "0001"
        );
        assert_eq!(
            resp.headers().get("ce-type").unwrap().to_str().unwrap(),
            "example.test"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost"
        );
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
        assert_eq!(
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );

        let bytes = test::load_body(resp.into_body()).await.unwrap();
        assert_eq!(j.to_string().as_bytes(), bytes.as_ref())
    }
}
