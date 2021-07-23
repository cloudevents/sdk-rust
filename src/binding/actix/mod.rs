//! This module integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [Actix web](https://docs.rs/actix-web/) to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent:
//!
//! ```
//! use cloudevents::binding::actix::HttpRequestExt;
//! use actix_web::{HttpRequest, web, post};
//!
//! #[post("/")]
//! async fn post_event(req: HttpRequest, payload: web::Payload) -> Result<String, actix_web::Error> {
//!     let event = req.to_event(payload).await?;
//!     println!("Received Event: {:?}", event);
//!     Ok(format!("{:?}", event))
//! }
//! ```
//!
//! To serialize a CloudEvent to an HTTP response:
//!
//! ```
//! use cloudevents::binding::actix::HttpResponseBuilderExt;
//! use actix_web::{HttpRequest, web, get, HttpResponse};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//!
//! #[get("/")]
//! async fn get_event() -> Result<HttpResponse, actix_web::Error> {
//!     HttpResponse::Ok()
//!         .event(
//!             EventBuilderV10::new()
//!                 .id("0001")
//!                 .ty("example.test")
//!                 .source("http://localhost/")
//!                 .data("application/json", json!({"hello": "world"}))
//!                 .build()
//!                 .expect("No error while building the event"),
//!         )
//! }
//! ```

#![deny(rustdoc::broken_intra_doc_links)]

mod server_request;
mod server_response;

pub use server_request::request_to_event;
pub use server_request::HttpRequestExt;
pub use server_response::event_to_response;
pub use server_response::HttpResponseBuilderExt;
