//! This module provides bindings between [cloudevents-sdk](https://docs.rs/cloudevents-sdk) and [nats](https://docs.rs/nats)
//! ## Examples
//! Deserialize [nats::Message](https://docs.rs/nats/0.21.0/nats/struct.Message.html) into [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html)
//! ```no_run
//!     use nats_lib as nats;
//!     use cloudevents::binding::nats::MessageExt;
//!     use futures::StreamExt;
//!     
//!     #[tokio::main]
//!     async fn main() {
//!       let nc = nats::connect("localhost:4222").await.unwrap();
//!       let mut sub = nc.subscribe("test").await.unwrap();
//!       
//!       // Process messages one at a time
//!       sub.for_each_concurrent(1, |nats_message| async move {
//!         let cloud_event = nats_message.to_event().unwrap();
//!         println!("{}", cloud_event.to_string());
//!       }).await;
//!     }
//! ```
//!
//! Serialize [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html) into [NatsCloudEvent] and publish to nats subject
//! ```no_run
//!     use nats_lib as nats;
//!     use cloudevents::binding::nats::NatsCloudEvent;
//!     use cloudevents::{EventBuilder, EventBuilderV10, Event};
//!     use serde_json::json;
//!
//!     #[tokio::main]
//!     async fn main() {
//!       let nc = nats::connect("localhost:4222").await.unwrap();
//!
//!       let event = EventBuilderV10::new()
//!           .id("123".to_string())
//!           .ty("example.test")
//!           .source("http://localhost/")
//!           .data("application/json", json!({"hello": "world"}))
//!           .build()
//!           .unwrap();
//!
//!       let nats_payload = NatsCloudEvent::from_event(event).unwrap();
//!       nc.publish("whatever.subject.you.like", nats_payload.payload.into()).await.unwrap();
//!     }
//! ```
mod deserializer;
mod serializer;

pub use deserializer::MessageExt;
pub use serializer::NatsCloudEvent;
