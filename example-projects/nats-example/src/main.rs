use std::error::Error;

use cloudevents::binding::nats::{MessageExt, NatsCloudEvent};
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use serde_json::json;
use futures::StreamExt;

/// First spin up a nats server i.e.
/// ```bash
/// docker run -p 4222:4222 -ti nats:latest
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = async_nats::connect("localhost:4222").await?;

    let event = EventBuilderV10::new()
        .id("123".to_string())
        .ty("example.test")
        .source("http://localhost/")
        .data("application/json", json!({"hello": "world"}))
        .build()
        .unwrap();

    let n_msg = NatsCloudEvent::from_event(event).unwrap();

    let mut sub = client.subscribe("test").await?;

    let receive_task = tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            match msg.to_event() {
                Ok(evt) => Ok(evt),
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err("No event received".to_string())
        }
    });

    client.publish("test", n_msg.payload.into()).await?;

    let maybe_event = receive_task.await?;

    if let Ok(evt) = maybe_event {
        println!("{}", evt.to_string());
    } else {
        println!("{}", maybe_event.unwrap_err());
    }

    Ok(())
}
