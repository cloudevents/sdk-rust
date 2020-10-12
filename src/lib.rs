//! This crate implements the [CloudEvents](https://cloudevents.io/) Spec for Rust.
//!
//! ```
//! use cloudevents::{EventBuilder, AttributesReader, EventBuilderV10};
//! use chrono::Utc;
//! use url::Url;
//!
//! let event = EventBuilderV10::new()
//!     .id("my_event.my_application")
//!     .source("http://localhost:8080")
//!     .ty("example.demo")
//!     .time(Utc::now())
//!     .build()
//!     .unwrap();
//!
//! println!("CloudEvent Id: {}", event.id());
//! println!("CloudEvent Time: {}", event.time().unwrap());
//! ```
//!
//! If you're looking for Protocol Binding implementations, look at crates:
//!
//! * [cloudevents-sdk-actix-web](https://docs.rs/cloudevents-sdk-actix-web): Integration with [Actix Web](https://github.com/actix/actix-web)
//! * [cloudevents-sdk-reqwest](https://docs.rs/cloudevents-sdk-reqwest): Integration with [reqwest](https://github.com/seanmonstar/reqwest)
//!

extern crate serde;
extern crate serde_json;
extern crate serde_value;
extern crate snafu;

/// Provides [`Event`] data structure, [`EventBuilder`] and other facilities to work with [`Event`]
pub mod event;
/// Provides facilities to implement Protocol Bindings
pub mod message;

pub use event::Event;
pub use event::{AttributesReader, AttributesWriter};
pub use event::{EventBuilder, EventBuilderV03, EventBuilderV10};
