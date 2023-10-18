//! This module integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [Warp web service framework](https://docs.rs/warp/)
//! to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent
//!
//! To echo events:
//!
//! ```
//! # use warp_lib as warp;
//! use warp::{Filter, Reply};
//! use cloudevents::binding::warp::reply::from_event;
//! use cloudevents::binding::warp::filter::to_event;
//!
//! let routes = warp::any()
//!     // extracting event from request
//!     .and(to_event())
//!     // returning event back
//!     .map(|event| from_event(event));
//!
//! warp::serve(routes).run(([127, 0, 0, 1], 3030));
//! ```
//!
//! To create event inside request handlers and send them as responses:
//!
//! ```
//! # use warp_lib as warp;
//! use cloudevents::{Event, EventBuilder, EventBuilderV10};
//! use http::StatusCode;
//! use serde_json::json;
//! use warp::{Filter, Reply};
//! use cloudevents::binding::warp::reply::from_event;
//!
//! let routes = warp::any().map(|| {
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
//!         .build();
//!
//!     match event {
//!         Ok(event) => from_event(event),
//!         Err(e) => warp::reply::with_status(
//!             e.to_string(),
//!             StatusCode::INTERNAL_SERVER_ERROR,
//!         ).into_response(),
//!     }
//! });
//!
//! warp::serve(routes).run(([127, 0, 0, 1], 3030));
//! ```

pub mod filter;
pub mod reply;
