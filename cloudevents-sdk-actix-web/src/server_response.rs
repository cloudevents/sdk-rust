use actix_web::http::{HeaderName, HeaderValue};
use actix_web::HttpResponse;
use cloudevents::message::{BinaryDeserializer, BinarySerializer, StructuredSerializer, MessageAttributeValue, SerializationResult, Error};
use cloudevents::Event;
use actix_web::web::BytesMut;
use cloudevents::event::SpecVersion;
use super::headers;
use std::io::Read;
use std::str::FromStr;
use actix_web::dev::HttpResponseBuilder;

struct HttpResponseSerializer {
    builder: HttpResponseBuilder
}

impl BinarySerializer<HttpResponse> for HttpResponseSerializer {
    fn set_spec_version(&mut self, spec_version: SpecVersion) -> SerializationResult {
        self
            .builder
            .set_header(
                headers::SPEC_VERSION_HEADER.clone(),
                str_to_header_value!(spec_version.as_str())?
            );
        SerializationResult::Ok(())
    }

    fn set_attribute(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self
            .builder
            .set_header(
                headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
                str_to_header_value!(value.to_string().as_str())?
            );
        SerializationResult::Ok(())
    }

    fn set_extension(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self
            .builder
            .set_header(
                attribute_name_to_header!(name)?,
                str_to_header_value!(value.to_string().as_str())?
            );
        SerializationResult::Ok(())
    }

    fn end_with_data<R: Read>(mut self, mut reader: R) -> Result<HttpResponse, Error> {
        let mut b = BytesMut::new();
        reader.read(&mut b)?;
        Ok(
            self
                .builder
                .body(b.freeze())
        )
    }

    fn end(mut self) -> Result<HttpResponse, Error> {
        Ok(self.builder.finish())
    }
}

impl StructuredSerializer<HttpResponse> for HttpResponseSerializer {
    fn set_structured_event<R: Read>(mut self, mut reader: R) -> Result<HttpResponse, Error> {
        let mut b = BytesMut::new();
        reader.read(&mut b)?;
        Ok(
            self.builder
                .set_header(actix_web::http::header::CONTENT_TYPE, headers::CLOUDEVENTS_JSON_HEADER.clone())
                .body(actix_web::body::Body::Bytes(b.freeze()))
        )
    }
}

pub async fn event_to_response(event: Event, response: HttpResponseBuilder) -> Result<HttpResponse, actix_web::error::Error> {
    BinaryDeserializer::deserialize_binary(event, HttpResponseSerializer { builder: response })
        .map_err(actix_web::error::ErrorBadRequest)
}