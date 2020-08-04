//! This crate integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [reqwest](https://docs.rs/reqwest/) to easily send and receive CloudEvents.
//!
//! ```
//! use cloudevents_sdk_reqwest::{RequestBuilderExt, ResponseExt};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//!
//! # async fn example() {
//! let client = reqwest::Client::new();
//!
//! // Prepare the event to send
//! let event_to_send = EventBuilderV10::new()
//!     .id("0001")
//!     .ty("example.test")
//!     .source("http://localhost/")
//!     .data("application/json", json!({"hello": "world"}))
//!     .build()
//!     .expect("No error while building the event");
//!
//! // Send request
//! let response = client.post("http://localhost")
//!   .event(event_to_send)
//!   .expect("Error while serializing the event")
//!   .send().await
//!   .expect("Error while sending the request");
//! // Parse response as event
//! let received_event = response
//!   .into_event().await
//!   .expect("Error while deserializing the response");
//! # }
//! ```
//!
//! Check out the cloudevents-sdk docs for more details on how to use [`cloudevents::Event`]: https://docs.rs/cloudevents-sdk

#[macro_use]
mod headers;
mod client_request;
mod client_response;

pub use client_request::event_to_request;
pub use client_request::RequestBuilderExt;
pub use client_request::RequestSerializer;
pub use client_response::response_to_event;
pub use client_response::ResponseDeserializer;
pub use client_response::ResponseExt;
