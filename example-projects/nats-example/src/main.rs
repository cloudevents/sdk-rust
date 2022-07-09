use std::{error::Error, thread};

use cloudevents::binding::nats::{MessageExt, NatsCloudEvent};
use cloudevents::{EventBuilder, EventBuilderV10, Event};
use serde_json::json;

/// First spin up a nats server i.e. 
/// ```bash
/// docker run -p 4222:4222 -ti nats:latest
/// ```
fn main() -> Result<(), Box<dyn Error>> {
    let nc = nats::connect("localhost:4222").unwrap();

    let event = EventBuilderV10::new()
        .id("123".to_string())
        .ty("example.test")
        .source("http://localhost/")
        .data("application/json", json!({"hello": "world"}))
        .build()
        .unwrap();

    let n_msg = NatsCloudEvent::from_event(event).unwrap();

    let sub = nc.subscribe("test").unwrap();

    let t = thread::spawn(move || -> Result<Event, String> {
        match sub.next() {
            Some(msg) => match msg.to_event() {
                Ok(evt) => {
                    Ok(evt)
                }
                Err(e) => Err(e.to_string()),
            },
            None => Err("Unsubed or disconnected".to_string()),
        }
    });
    
    nc.publish("test", n_msg)?;
    
    let maybe_event = t.join().unwrap();

    if let Ok(evt) = maybe_event {
        println!("{}", evt.to_string());
    } else {
        println!("{}", maybe_event.unwrap_err().to_string());
    }

    Ok(())

}
