use cloudevents::binding::reqwest::RequestBuilderExt;
use cloudevents::{EventBuilder, EventBuilderV10};
use wasm_bindgen::prelude::*;
use uuid::Uuid;

#[wasm_bindgen]
pub async fn run(
    target: String,
    ty: String,
    datacontenttype: String,
    data: String,
) -> Result<(), JsValue> {
    let event = EventBuilderV10::new()
        .id(&Uuid::new_v4().to_hyphenated().to_string())
        .ty(ty)
        .source("http://localhost/")
        .data(datacontenttype, data)
        .build()
        .unwrap();

    println!("Going to send event: {:?}", event);

    reqwest::Client::new()
        .post(&target)
        .event(event)
        .map_err(|e| e.to_string())?
        .header("Access-Control-Allow-Origin", "*")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
