//! This module provides bindings between [cloudevents-sdk](https://docs.rs/cloudevents-sdk) and [nats](https://docs.rs/nats)
//! 
//! Deserialize [nats::Message] into [Event]
//! ```rust
//!     let nc = nats::connect("localhost:4222").unwrap();
//!     let sub = nc.subscribe("test").unwrap();
//!     let nats_message = sub.next().unwrap();
//!     let cloud_event = nats_message.to_event().unwrap();
//!     println!("{}", evt.to_string());
//! ```
//! 
//! Serialize [Event] into [NatsCloudEvent] and publish to nats subject
//! ```rust
//!     let nc = nats::connect("localhost:4222").unwrap();
//! 
//!     let event = EventBuilderV10::new()
//!         .id("123".to_string())
//!         .ty("example.test")
//!         .source("http://localhost/")
//!         .data("application/json", json!({"hello": "world"}))
//!         .build()
//!         .unwrap();
//! 
//!     nc.publish("whatever.subject.you.like", NatsCloudEvent::from_event(event).unwrap());
//! ```
mod serializer;
mod deserializer;

pub use serializer::NatsCloudEvent;
pub use deserializer::{MessageExt};