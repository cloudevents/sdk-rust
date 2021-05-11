use cloudevents::{EventBuilder, EventBuilderV10, Event};
use cloudevents_sdk_tide::*;
use tide::{Response, Request, Body};
 
 pub async fn get(_req: Request<()>) -> tide::Result {
     Ok(Response::new(200).event(
             EventBuilderV10::new()
                 .id("0001")
                 .ty("example.test")
                 .source("http://localhost/")
                 .data("text/xml", "<xml data=\"hello\" />".as_bytes().to_vec())
                 .build()
                 .expect("No error while building the event"),
         ).await?
     )
 }

 pub async fn post(req: Request<()>) -> tide::Result {
     let evtresp: Event = req.to_event().await?;
     let response = Response::builder(200)
     .body(Body::from_json(&evtresp)?)
     .build();
     Ok(response)
 }

 //Test post with 
 // curl -H "Content-Type:text/plain" -H "ce-specversion:1.0" -H "ce-id:0001" -H "ce-source:http://localhost"  -H "ce-type:example.test" -d "hello" http://127.0.0.1:8080/ 
 #[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    let mut index = app.at("/");
    index.get(get);
    index.post(post);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}