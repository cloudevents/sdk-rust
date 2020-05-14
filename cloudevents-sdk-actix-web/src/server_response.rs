use actix_web::http::{HeaderName, HeaderValue};
use actix_web::{HttpMessage, HttpRequest, web, HttpResponse};
use cloudevents::message::{BinaryDeserializer, BinarySerializer, DeserializationResult, Encoding, MessageDeserializer, StructuredDeserializer, StructuredSerializer, MessageAttributeValue, SerializationResult};
use cloudevents::{Event, message};
use actix_web::web::{BytesMut, Bytes};
use futures::StreamExt;
use bytes::buf::BufExt;
use cloudevents::event::SpecVersion;
use std::convert::TryFrom;
use super::headers;
use std::io::Read;
use std::borrow::BorrowMut;
use actix_web::guard::Header;
use std::str::FromStr;

struct HttpResponseSerializer<'a> {
    req: &'a mut HttpResponse,
}

impl <'a> BinarySerializer for HttpResponseSerializer<'a> {
    fn set_spec_version(&mut self, spec_version: SpecVersion) -> SerializationResult {
        self
            .req
            .headers_mut()
            .append(
            headers::SPEC_VERSION_HEADER.clone(),
            str_to_header_value!(spec_version.as_str())?
        );
        SerializationResult::Ok(())
    }

    fn set_attribute(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self
            .req
            .headers_mut()
            .append(
                headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
                str_to_header_value!(value.to_string().as_str())?
            );
        SerializationResult::Ok(())
    }

    fn set_extension(&mut self, name: &str, value: MessageAttributeValue) -> SerializationResult {
        self
            .req
            .headers_mut()
            .append(
                attribute_name_to_header!(name)?,
                str_to_header_value!(value.to_string().as_str())?
            );
        SerializationResult::Ok(())
    }

    fn set_body<R: Read>(&mut self, reader: R) -> SerializationResult {
        self.req.set_body(reader)
    }
}

impl <'a> StructuredSerializer for HttpResponseSerializer<'a> {
    fn set_structured_event<R: Read>(mut self, reader: R) -> SerializationResult {
        unimplemented!()
    }
}

pub async fn event_to_response(event: Event, response: &mut HttpResponse) -> Result<(), actix_web::error::Error> {
    let mut serializer = HttpResponseSerializer { req: response };
    BinaryDeserializer::deserialize_binary(event, &mut serializer)
        .map_err(actix_web::error::ErrorBadRequest)
}