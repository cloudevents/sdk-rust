use axum_lib as axum;

use axum::{body::Body, http::Response};
use std::cell::Cell;

use crate::binding::http::{Builder, Serializer};
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

pub fn event_to_response(event: Event) -> std::result::Result<Response<Body>, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}
