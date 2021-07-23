use warp_lib as warp;

use crate::binding::{
    http::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{
    BinaryDeserializer, BinarySerializer, Error, MessageAttributeValue, Result,
    StructuredSerializer,
};
use crate::{str_to_header_value, Event};

use warp::hyper::Body;
use warp::reply::Response;

use http::response::Builder;

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
        self.builder = self
            .builder
            .header(SPEC_VERSION_HEADER, str_to_header_value!(spec_version)?);
        Ok(self)
    }

    fn set_attribute(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self
            .builder
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn set_extension(mut self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder = self
            .builder
            .header(&header_prefix(name), str_to_header_value!(value)?);
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
            .header(http::header::CONTENT_TYPE, CLOUDEVENTS_JSON_HEADER)
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
}

pub fn event_to_response(event: Event) -> std::result::Result<Response, Error> {
    BinaryDeserializer::deserialize_binary(event, ResponseSerializer::new())
}
