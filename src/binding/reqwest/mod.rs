//! This module integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [reqwest](https://docs.rs/reqwest/) to easily send and receive CloudEvents.
//!
//! ```
//! # use reqwest_lib as reqwest;
//! use cloudevents::binding::reqwest::{RequestBuilderExt, ResponseExt};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = reqwest::Client::new();
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
//! let response = client.post("http://localhost")
//!   .event(event_to_send)?
//!   .send().await?;
//! // Parse response as event
//! let received_event = response
//!   .into_event().await?;
//! # Ok(())
//! # }
//! ```

#![deny(broken_intra_doc_links)]

mod client_request;
mod client_response;

pub use client_request::event_to_request;
pub use client_request::RequestBuilderExt;
pub use client_request::RequestSerializer;
pub use client_response::response_to_event;
pub use client_response::ResponseExt;
