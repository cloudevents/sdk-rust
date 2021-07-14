use crate::binding::{
    attribute_header,
    http::{PREFIX, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use crate::Event;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::{HeaderValue, StatusCode};
use actix_web::HttpResponse;
use async_trait::async_trait;
use futures::future::LocalBoxFuture;
use futures::FutureExt;

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
        self.builder
            .set_header(SPEC_VERSION_HEADER, str_to_header_value!(spec_version)?);
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.set_header(
            &attribute_header(PREFIX, name),
            str_to_header_value!(value)?,
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.set_header(
            &attribute_header(PREFIX, name),
            str_to_header_value!(value)?,
        );
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
            .set_header(
                actix_web::http::header::CONTENT_TYPE,
                CLOUDEVENTS_JSON_HEADER,
            )
            .body(bytes))
    }
}

/// Method to fill an [`HttpResponseBuilder`] with an [`Event`].
pub async fn event_to_response(
    event: Event,
    response: HttpResponseBuilder,
) -> std::result::Result<HttpResponse, actix_web::error::Error> {
    BinaryDeserializer::deserialize_binary(event, HttpResponseSerializer::new(response))
        .map_err(actix_web::error::ErrorBadRequest)
}

/// So that an actix-web handler may return an Event
impl actix_web::Responder for Event {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, std::result::Result<HttpResponse, Self::Error>>;

    fn respond_to(self, _: &actix_web::HttpRequest) -> Self::Future {
        async { HttpResponse::build(StatusCode::OK).event(self).await }.boxed_local()
    }
}

/// Extension Trait for [`HttpResponseBuilder`] which acts as a wrapper for the function [`event_to_response()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
#[async_trait(?Send)]
pub trait HttpResponseBuilderExt: private::Sealed {
    /// Fill this [`HttpResponseBuilder`] with an [`Event`].
    async fn event(
        self,
        event: Event,
    ) -> std::result::Result<HttpResponse, actix_web::error::Error>;
}

#[async_trait(?Send)]
impl HttpResponseBuilderExt for HttpResponseBuilder {
    async fn event(
        self,
        event: Event,
    ) -> std::result::Result<HttpResponse, actix_web::error::Error> {
        event_to_response(event, self).await
    }
}

// Sealing the HttpResponseBuilderExt
mod private {
    pub trait Sealed {}
    impl Sealed for actix_web::dev::HttpResponseBuilder {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{EventBuilder, EventBuilderV10};
    use actix_web::http::StatusCode;
    use actix_web::test;
    use futures::TryStreamExt;
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
            .await
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

        let mut resp = HttpResponseBuilder::new(StatusCode::OK)
            .event(input)
            .await
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

        let bytes = test::load_stream(resp.take_body().into_stream())
            .await
            .unwrap();
        assert_eq!(j.to_string().as_bytes(), bytes.as_ref())
    }
}
