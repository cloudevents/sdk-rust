#[cfg(feature = "axum")]
use bytes::Bytes;
use http::Response;
#[cfg(feature = "axum")]
use http_1_1 as http;
#[cfg(feature = "axum")]
use http_body_util::Full;
#[cfg(not(feature = "axum"))]
use hyper::body::Body;
use std::cell::Cell;

use crate::binding::http::{Builder, Serializer};
use crate::message::{BinaryDeserializer, Error, Result};
use crate::Event;
#[cfg(feature = "axum")]
use std::convert::Infallible;
#[cfg(feature = "axum")]
type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, Infallible>;

struct Adapter {
    builder: Cell<http::response::Builder>,
}

#[cfg(not(feature = "axum"))]
impl Builder<Response<Body>> for Adapter {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.builder.set(self.builder.take().header(key, value));
    }
    fn body(&mut self, bytes: Vec<u8>) -> Result<Response<Body>> {
        self.builder
            .take()
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
    fn finish(&mut self) -> Result<Response<Body>> {
        self.body(Vec::new())
    }
}

#[cfg(feature = "axum")]
impl Builder<Response<BoxBody>> for Adapter {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.builder.set(self.builder.take().header(key, value));
    }

    fn body(&mut self, bytes: Vec<u8>) -> Result<Response<BoxBody>> {
        self.builder
            .take()
            .body(BoxBody::new(Full::from(bytes)))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }

    fn finish(&mut self) -> Result<Response<BoxBody>> {
        self.body(Vec::new())
    }
}

#[cfg(not(feature = "axum"))]
pub fn to_response(event: Event) -> std::result::Result<Response<Body>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}

#[cfg(feature = "axum")]
pub fn to_response(event: Event) -> std::result::Result<Response<BoxBody>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}
