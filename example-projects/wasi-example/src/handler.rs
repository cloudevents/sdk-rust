use cloudevents::{event::Data, Event, EventBuilder, EventBuilderV10};
use log::info;
use serde_json::{from_slice, from_str, json};

pub async fn handle_event(event: Event) -> Result<Event, anyhow::Error> {
    info!("event: {}", event);

    let input = match event.data() {
        Some(Data::Binary(v)) => from_slice(v)?,
        Some(Data::String(v)) => from_str(v)?,
        Some(Data::Json(v)) => v.to_owned(),
        None => json!({ "name": "default" }),
    };

    EventBuilderV10::from(event)
        .source("func://handler")
        .ty("func.example")
        .data("application/json", json!({ "hello": input["name"] }))
        .build()
        .map_err(|err| err.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn post_test() -> Result<(), anyhow::Error> {
        let reqevt = Event::default();
        let respevt = handle_event(reqevt).await?;
        let output = match respevt.data() {
            Some(Data::Binary(v)) => from_slice(v)?,
            Some(Data::String(v)) => from_str(v)?,
            Some(Data::Json(v)) => v.to_owned(),
            None => json!({ "name": "default" }),
        };
        assert_eq!(output, json!({ "hello": "default" }));
        Ok(())
    }
}
