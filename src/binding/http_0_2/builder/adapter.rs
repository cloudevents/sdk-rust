use http::Response;
use http_0_2 as http;
use hyper::body::Body;
use std::cell::Cell;

#[cfg(not(target_os = "wasi"))]
use hyper_0_14 as hyper;

#[cfg(target_os = "wasi")]
use hyper;

use crate::binding::http_0_2::{Builder, Serializer};
use crate::message::{BinaryDeserializer, Error, Result};
use crate::Event;

struct Adapter {
    builder: Cell<http::response::Builder>,
}

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

pub fn to_response(event: Event) -> std::result::Result<Response<Body>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}
