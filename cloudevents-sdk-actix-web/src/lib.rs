//! This crate integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [Actix web](https://docs.rs/actix-web/) to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent:
//!
//! ```
//! use cloudevents_sdk_actix_web::RequestExt;
//! use actix_web::{HttpRequest, web, post};
//!
//! #[post("/")]
//! async fn post_event(req: HttpRequest, payload: web::Payload) -> Result<String, actix_web::Error> {
//!     let event = req.into_event(payload).await?;
//!     println!("Received Event: {:?}", event);
//!     Ok(format!("{:?}", event))
//! }
//! ```
//!
//! To serialize a CloudEvent to an HTTP response:
//!
//! ```
//! use cloudevents_sdk_actix_web::HttpResponseBuilderExt;
//! use actix_web::{HttpRequest, web, get, HttpResponse};
//! use cloudevents::{EventBuilderV10, EventBuilder};
//! use serde_json::json;
//!
//! #[get("/")]
//! async fn get_event() -> Result<HttpResponse, actix_web::Error> {
//!     Ok(HttpResponse::Ok()
//!         .event(
//!             EventBuilderV10::new()
//!                 .id("0001")
//!                 .ty("example.test")
//!                 .source("http://localhost/")
//!                 .data("application/json", json!({"hello": "world"}))
//!                 .build()
//!                 .expect("No error while building the event"),
//!         )
//!         .await?
//!     )
//! }
//! ```
//!
//! Check out the cloudevents-sdk docs for more details on how to use [`cloudevents::Event`]: https://docs.rs/cloudevents-sdk

#[macro_use]
mod headers;
mod server_request;
mod server_response;

pub use server_request::request_to_event;
pub use server_request::HttpRequestDeserializer;
pub use server_request::RequestExt;
pub use server_response::event_to_response;
pub use server_response::HttpResponseBuilderExt;
pub use server_response::HttpResponseSerializer;
