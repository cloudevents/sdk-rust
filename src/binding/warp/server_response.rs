use warp_lib as warp;

use super::headers;

use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Error, MessageAttributeValue, Result,
    StructuredSerializer,
};
use crate::Event;

use warp::http::HeaderValue;
use warp::hyper::Body;
use warp::reply::Response;

use http::header::HeaderName;
use http::response::Builder;

use std::{convert::TryFrom, str::FromStr};

pub struct ResponseSerializer {
    builder: Builder,
}

impl ResponseSerializer {
    fn new() -> Self {
        ResponseSerializer {
            builder: http::Response::builder(),
        }
    }
}

impl BinarySerializer<Response> for ResponseSerializer {
    fn set_spec_version(mut self, spec_version: SpecVersion) -> Result<Self> {
        self.builder = self.builder.header(
            headers::SPEC_VERSION_HEADER.clone(),
            HeaderValue::try_from(spec_version.to_string().as_str()).map_err(|e| {
                crate::message::Error::Other {
                    source: Box::new(e),
                }
            })?,
        );
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self.builder.header(
            headers::ATTRIBUTES_TO_HEADERS.get(name).unwrap().clone(),
            HeaderValue::try_from(value.to_string().as_str()).map_err(|e| {
                crate::message::Error::Other {
                    source: Box::new(e),
                }
            })?,
        );
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self.builder.header(
            attribute_name_to_header!(name)?,
            HeaderValue::try_from(value.to_string().as_str()).map_err(|e| {
                crate::message::Error::Other {
                    source: Box::new(e),
                }
            })?,
        );
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<Response> {
        self.builder
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }

    fn end(self) -> Result<Response> {
        self.builder
            .body(Body::empty())
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
}

impl StructuredSerializer<Response> for ResponseSerializer {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<Response> {
        self.builder
            .header(
                http::header::CONTENT_TYPE,
                headers::CLOUDEVENTS_JSON_HEADER.clone(),
            )
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
}

pub fn event_to_response(event: Event) -> std::result::Result<Response, Error> {
    BinaryDeserializer::deserialize_binary(event, ResponseSerializer::new())
}
