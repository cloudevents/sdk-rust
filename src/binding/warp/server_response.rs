use std::cell::Cell;

use warp_lib as warp;

use crate::binding::http::{Builder, Serializer};
use crate::message::{BinaryDeserializer, Error, Result};
use crate::Event;

use warp::hyper::Body;
use warp::reply::Response;

struct Adapter {
    builder: Cell<http::response::Builder>,
}

impl Builder<Response> for Adapter {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.builder.set(self.builder.take().header(key, value));
    }
    fn body(&mut self, bytes: Vec<u8>) -> Result<Response> {
        self.builder
            .take()
            .body(Body::from(bytes))
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    }
    fn finish(&mut self) -> Result<Response> {
        self.body(Vec::new())
    }
}

pub fn event_to_response(event: Event) -> std::result::Result<Response, Error> {
    BinaryDeserializer::deserialize_binary(
        event,
        Serializer::new(Adapter {
            builder: Cell::new(http::Response::builder()),
        }),
    )
}
