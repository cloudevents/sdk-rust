#[cfg(any(feature = "hyper", feature = "hyper-1-3"))]
pub mod adapter;

use crate::message::Result;

use http;

pub trait Builder<R> {
    fn header(&mut self, key: &str, value: http::header::HeaderValue);
    fn body(&mut self, bytes: Vec<u8>) -> Result<R>;
    fn finish(&mut self) -> Result<R>;
}
