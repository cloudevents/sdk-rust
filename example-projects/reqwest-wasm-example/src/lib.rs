use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn run(target: String, ty: String, datacontenttype: String, data: String) -> Result<(), String> {
    let event = cloudevents::EventBuilder::new()
        .ty(ty)
        .data(datacontenttype, data)
        .build();

    println!("Going to send event: {:?}", event);

    cloudevents_sdk_reqwest::event_to_request(event, reqwest::Client::new().post(&target))
        .map_err(|e| e.to_string())?
        .header("Access-Control-Allow-Origin", "*")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}