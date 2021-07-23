//! This crate implements the [CloudEvents](https://cloudevents.io/) Spec for Rust.
//!
//! ```
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use cloudevents::{EventBuilder, AttributesReader, EventBuilderV10};
//! use chrono::{Utc, DateTime};
//! use url::Url;
//!
//! let event = EventBuilderV10::new()
//!     .id("my_event.my_application")
//!     .source("http://localhost:8080")
//!     .ty("example.demo")
//!     .time(Utc::now())
//!     .build()?;
//!
//! println!("CloudEvent Id: {}", event.id());
//! match event.time() {
//!     Some(t) => println!("CloudEvent Time: {}", t),
//!     None => println!("CloudEvent Time: None")
//! }
//! # Ok(())
//! # }
//! ```
//!
//! This crate includes:
//!
//! * The [`Event`] data structure, to represent CloudEvent (version 1.0 and 0.3)
//! * The [`EventBuilder`] trait and implementations, to create [`Event`] instances
//! * The implementation of [`serde::Serialize`] and [`serde::Deserialize`] for [`Event`] to serialize/deserialize CloudEvents to/from JSON
//! * Traits and utilities in [`message`] to implement Protocol Bindings
//! * Feature-guarded modules for various Protocol Binding implementations, e.g. actix, axum, reqwest, warp, rdkafka
//!
//! ## Feature flags
//!
//! Cloudevents uses a set of [feature flags] to conditionally compile
//! only the module associated with the Protocol Binding you need:
//!
//! - `actix`: Enables the [`binding::actix`] protocol binding module. This
//! extends the [`actix_web::HttpRequest`] with a
//! [`to_event`](binding::actix::HttpRequestExt::to_event) function, the
//! [`actix_web::HttpResponseBuilder`] with an
//! [`event`](binding::actix::HttpResponseBuilderExt::event) function,
//! and implementations for [`actix_web::FromRequest`] and
//! [`actix_web::Responder`] in order to take advantage of actix-web's
//! [Extractors] and [Responders]
//! - `reqwest`: Enables the [`binding::reqwest`] protocol binding module.
//! - `warp`: Enables the [`binding::warp`] protocol binding module.
//! - `axum`: Enables the [`binding::axum`] protocol binding module.
//! - `rdkafka`: Enables the [`binding::rdkafka`] protocol binding module to
//! seamlessly consume/produce cloudevents within Kafka messages.
//!
//! [feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
//! [Extractors]: https://actix.rs/docs/extractors/
//! [Responders]: https://actix.rs/docs/handlers/

#![doc(html_root_url = "https://docs.rs/cloudevents-sdk/0.4.0")]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod binding;
pub mod event;
pub mod message;

#[cfg(test)]
pub mod test;

pub use event::Data;
pub use event::Event;
pub use event::{AttributesReader, AttributesWriter};
pub use event::{EventBuilder, EventBuilderV03, EventBuilderV10};
