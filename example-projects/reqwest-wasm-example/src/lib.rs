use cloudevents::binding::reqwest::RequestBuilderExt;
use cloudevents::{EventBuilder, EventBuilderV10};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn run(
    target: String,
    ty: String,
    datacontenttype: String,
    data: String,
) -> Result<(), String> {
    let event = EventBuilderV10::new()
        .ty(ty)
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
