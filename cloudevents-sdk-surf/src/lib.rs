//! This crate integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [surf](https://docs.rs/surf/) to easily send and receive CloudEvents.
//!
//! ```
//! use cloudevents_sdk_surf::{RequestExt, ResponseExt};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//! use surf::{http, Url, Request};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = surf::Client::new();
//!
//! // Prepare the event to send
//! let event_to_send = EventBuilderV10::new()
//!     .id("0001")
//!     .ty("example.test")
//!     .source("http://localhost/")
//!     .data("application/json", json!({"hello": "world"}))
//!     .build()?;
//!
//! // Send request
//! let req = Request::new(http::Method::Post, Url::parse("http://localhost/").unwrap());
//! let evt = req.event(event_to_send).await.unwrap();
//! let client = surf::Client::new();
//! let response = client.send(evt).await.unwrap();
//! let received_event = response
//!   .to_event().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Check out the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) docs for more details on how to use [`cloudevents::Event`].

#![doc(html_root_url = "https://docs.rs/cloudevents-sdk-surf/0.3.1")]
#![deny(broken_intra_doc_links)]

#[macro_use]
mod headers;
mod client_request;
mod client_response;

pub use client_request::event_to_request;
pub use client_request::RequestExt;
pub use client_request::RequestSerializer;
pub use client_response::response_to_event;
pub use client_response::ResponseDeserializer;
pub use client_response::ResponseExt;
