use super::headers;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::{HeaderName, HeaderValue};
use actix_web::HttpResponse;
use async_trait::async_trait;
use cloudevents::event::SpecVersion;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, MessageAttributeValue, Result, StructuredSerializer,
};
use cloudevents::Event;
use std::str::FromStr;

/// Wrapper for [`HttpResponseBuilder`] that implements [`StructuredSerializer`] and [`BinarySerializer`]
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
        self.builder.set_header(
            headers::SPEC_VERSION_HEADER.clone(),
            str_to_header_value!(spec_version.as_str())?,
        );
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.set_header(
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            str_to_header_value!(value.to_string().as_str())?,
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder.set_header(
            attribute_name_to_header!(name)?,
            str_to_header_value!(value.to_string().as_str())?,
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
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
            )
            .body(bytes))
    }
}

/// Method to fill an [`HttpResponseBuilder`] with an [`Event`]
pub async fn event_to_response(
    event: Event,
    response: HttpResponseBuilder,
) -> std::result::Result<HttpResponse, actix_web::error::Error> {
    BinaryDeserializer::deserialize_binary(event, HttpResponseSerializer::new(response))
        .map_err(actix_web::error::ErrorBadRequest)
}

#[async_trait(?Send)]
pub trait EventExt {
    async fn into_response(
        self,
        response: HttpResponseBuilder,
    ) -> std::result::Result<HttpResponse, actix_web::error::Error>;
}

#[async_trait(?Send)]
impl EventExt for Event {
    async fn into_response(
        self,
        response: HttpResponseBuilder,
    ) -> std::result::Result<HttpResponse, actix_web::error::Error> {
        event_to_response(self, response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    use actix_web::http::StatusCode;
    use actix_web::test;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use futures::TryStreamExt;
    use serde_json::json;
    use std::str::FromStr;

    #[actix_rt::test]
    async fn test_response() {
        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost/").unwrap())
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = input
            .into_response(HttpResponseBuilder::new(StatusCode::OK))
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
            .source(Url::from_str("http://localhost").unwrap())
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let mut resp = input
            .into_response(HttpResponseBuilder::new(StatusCode::OK))
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
