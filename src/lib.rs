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
//!
//! If you're looking for Protocol Binding implementations, look at crates:
//!
//! * [cloudevents-sdk-actix-web](https://docs.rs/cloudevents-sdk-actix-web): Integration with [Actix Web](https://github.com/actix/actix-web)
//! * [cloudevents-sdk-reqwest](https://docs.rs/cloudevents-sdk-reqwest): Integration with [reqwest](https://github.com/seanmonstar/reqwest)
//! * [cloudevents-sdk-rdkafka](https://docs.rs/cloudevents-sdk-rdkafka): Integration with [rdkafka](https://fede1024.github.io/rust-rdkafka)
//!

#![doc(html_root_url = "https://docs.rs/cloudevents-sdk/0.3.1")]
#![deny(broken_intra_doc_links)]

#[cfg(feature = "cloudevents-actix")]
pub mod actix;
#[cfg(feature = "cloudevents-rdkafka")]
pub mod rdkafka;
#[cfg(feature = "cloudevents-reqwest")]
pub mod reqwest;
#[cfg(feature = "cloudevents-warp")]
pub mod warp;

pub mod event;
pub mod message;

pub use event::Data;
pub use event::Event;
pub use event::{AttributesReader, AttributesWriter};
pub use event::{EventBuilder, EventBuilderV03, EventBuilderV10};
