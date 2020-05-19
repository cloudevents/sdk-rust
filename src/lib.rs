//! This crate implements the [CloudEvents](https://cloudevents.io/) Spec for Rust.
//!
//! ```
//! use cloudevents::{EventBuilder, AttributesReader};
//! use chrono::Utc;
//! use url::Url;
//!
//! let event = EventBuilder::v10()
//!     .id("my_event.my_application")
//!     .source(Url::parse("http://localhost:8080").unwrap())
//!     .time(Utc::now())
//!     .build();
//!
//! println!("CloudEvent Id: {}", event.get_id());
//! println!("CloudEvent Time: {}", event.get_time().unwrap());
//! ```
//!
//! If you're looking for Protocol Binding implementations, look at crates:
//!
//! * `cloudevents-sdk-actix-web`: Integration with [Actix Web](https://github.com/actix/actix-web)
//! * `cloudevents-sdk-reqwest`: Integration with [reqwest](https://github.com/seanmonstar/reqwest)
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
pub use event::EventBuilder;
pub use event::{AttributesReader, AttributesWriter};
