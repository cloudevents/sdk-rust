//! This module provides bindings between [cloudevents-sdk](https://docs.rs/cloudevents-sdk) and [nats](https://docs.rs/nats)
//! ## Examples
//! Deserialize [nats::Message](https://docs.rs/nats/latest/nats/struct.Message.html) into [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html)
//! ```rust
//!     use nats_lib as nats;
//!     use cloudevents::binding::nats::MessageExt;
//!     
//!     fn consume() {
//!       let nc = nats::connect("localhost:4222").unwrap();
//!       let sub = nc.subscribe("test").unwrap();
//!       let nats_message = sub.next().unwrap();
//!       let cloud_event = nats_message.to_event().unwrap();
//!
//!       println!("{}", cloud_event.to_string());
//!     }
//! ```
//!
//! Serialize [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html) into [NatsCloudEvent] and publish to nats subject
//! ```rust
//!     use nats_lib as nats;
//!     use cloudevents::binding::nats::NatsCloudEvent;
//!     use cloudevents::{EventBuilder, EventBuilderV10, Event};
//!     use serde_json::json;
//!
//!     fn publish() {
//!       let nc = nats::connect("localhost:4222").unwrap();
//!
//!       let event = EventBuilderV10::new()
//!           .id("123".to_string())
//!           .ty("example.test")
//!           .source("http://localhost/")
//!           .data("application/json", json!({"hello": "world"}))
//!           .build()
//!           .unwrap();
//!
//!       nc.publish("whatever.subject.you.like", NatsCloudEvent::from_event(event).unwrap());
//!     }
//! ```
mod deserializer;
mod serializer;

pub use deserializer::MessageExt;
pub use serializer::NatsCloudEvent;
