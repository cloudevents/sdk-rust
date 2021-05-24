//! This crate integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [tide](https://docs.rs/tide/) to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent:
//!
//! ```
//! use cloudevents::Event;
//! use cloudevents_sdk_tide::{ RequestExt };
//! use tide::{Request, Body, Response};
//!
//! pub async fn index(mut req: Request<()>) -> tide::Result {
//!     // The req headers should be set by the client but included here for clarity.
//!     req.insert_header("content-type", "application/json");
//!     req.insert_header("ce-specversion", "1.0");
//!     req.insert_header("ce-id", "0001");
//!     req.insert_header("ce-type", "example.test");
//!     req.insert_header("ce-source", "http://localhost/");
//!     let event : Event = req.to_event().await?;
//!     let resp = Response::builder(200).body(Body::from_json(&event)?).build();
//!     Ok(resp)
//! }
//!```
//!
//! To serialize a CloudEvent to an HTTP response:
//!
//! ```
//! use cloudevents_sdk_tide::ResponseExt;
//! use tide::{Request, Response, Result};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//!
//! pub async fn index(req: Request<()>) -> tide::Result {
//!     Ok(Response::new(200).event(
//!             EventBuilderV10::new()
//!                 .id("0001")
//!                 .ty("example.test")
//!                 .source("http://localhost/")
//!                 .data("application/json", json!({"hello": "world"}))
//!                 .build()
//!                 .expect("No error while building the event"),
//!         ).await?
//!     )
//! }
//! ```
//!
//! Check out the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) docs for more details on how to use [`cloudevents::Event`]

#![doc(html_root_url = "https://docs.rs/cloudevents-sdk-tide/0.0.1")]
#![deny(broken_intra_doc_links)]

#[macro_use]
mod headers;
mod server_request;
mod server_response;

pub use cloudevents::Event;
pub use server_request::request_to_event;
pub use server_request::RequestDeserializer;
pub use server_request::RequestExt;
pub use server_response::event_to_response;
pub use server_response::ResponseExt;
pub use server_response::ResponseSerializer;
