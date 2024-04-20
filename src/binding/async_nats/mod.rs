//! This module provides bindings between [cloudevents-sdk](https://docs.rs/cloudevents-sdk) and [async-nats](https://docs.rs/async-nats)
//! ## Examples
//! Deserialize [async_nats::Message](https://docs.rs/async-nats/latest/async_nats/message/struct.Message.html) into [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html)
//! ```
//!     use async_nats_lib as async_nats;
//!     use futures::StreamExt;
//!     use cloudevents::binding::async_nats::MessageExt;
//!
//!     async fn consume() {
//!       let nc = async_nats::connect("localhost:4222").await.unwrap();
//!       let mut  sub = nc.subscribe("test".to_string()).await.unwrap();
//!       let nats_message = sub.next().await.unwrap();
//!       let cloud_event = nats_message.to_event().unwrap();
//!
//!       println!("{}", cloud_event.to_string());
//!     }
//! ```
//!
//! Serialize [Event](https://docs.rs/cloudevents-sdk/latest/cloudevents/event/struct.Event.html) into [NatsCloudEvent] and publish to nats subject
//! ```
//!     use async_nats_lib as async_nats;
//!     use cloudevents::binding::nats::NatsCloudEvent;
//!     use cloudevents::{EventBuilder, EventBuilderV10, Event};
//!     use serde_json::json;
//!
//!     async fn publish() {
//!       let nc = async_nats::connect("localhost:4222").await.unwrap();
//!
//!       let event = EventBuilderV10::new()
//!           .id("123".to_string())
//!           .ty("example.test")
//!           .source("http://localhost/")
//!           .data("application/json", json!({"hello": "world"}))
//!           .build()
//!           .unwrap();
//!
//!       nc.publish("whatever.subject.you.like".to_string(), NatsCloudEvent::from_event(event).unwrap().payload.into()).await.unwrap();
//!     }
//! ```
mod deserializer;
mod serializer;

pub use deserializer::MessageExt;
pub use serializer::NatsCloudEvent;
