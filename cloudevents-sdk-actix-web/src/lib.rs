#![feature(str_strip)]

#[macro_use]
mod headers;
mod server_request;
mod server_response;

pub use server_request::*;
pub use server_response::*;
