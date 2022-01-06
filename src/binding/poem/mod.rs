//! This module integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with
//! [Poem](https://docs.rs/poem/) to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent
//!
//! To echo events:
//!
//! ```rust
//! use cloudevents::Event;
//! use poem_lib as poem;
//! use poem::{handler, Route, post};
//!
//! #[handler]
//! async fn index(event: Event) -> Event {
//!     println!("received cloudevent {}", &event);
//!     event
//! }
//!
//! let app = Route::new().at("/", post(index));
//! ```
//!
//! To create event inside request handlers and send them as responses:
//!
//! ```rust
//! use cloudevents::{Event, EventBuilder, EventBuilderV10};
//! use poem_lib as poem;
//! use poem::{handler, Route, post, Result};
//! use poem::error::InternalServerError;
//! use serde_json::json;
//!
//! #[handler]
//! async fn index() -> Result<Event> {
//!     let event = EventBuilderV10::new()
//!         .id("1")
//!         .source("url://example_response/")
//!         .ty("example.ce")
//!         .data(
//!             mime::APPLICATION_JSON.to_string(),
//!             json!({
//!                 "name": "John Doe",
//!                 "age": 43,
//!                 "phones": [
//!                     "+44 1234567",
//!                     "+44 2345678"
//!                 ]
//!             }),
//!         )
//!         .build()
//!         .map_err(InternalServerError)?;
//!     Ok(event)
//! }
//!
//! let app = Route::new().at("/", post(index));
//! ```

mod extractor;
mod response;
