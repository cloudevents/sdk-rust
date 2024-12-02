#[cfg(feature = "hyper-0-14")]
pub mod adapter;

use crate::message::Result;
use http_0_2 as http;

pub trait Builder<R> {
    fn header(&mut self, key: &str, value: http::header::HeaderValue);
    fn body(&mut self, bytes: Vec<u8>) -> Result<R>;
    fn finish(&mut self) -> Result<R>;
}
