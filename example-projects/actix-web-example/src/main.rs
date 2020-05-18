use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use cloudevents::EventBuilder;
use url::Url;
use std::str::FromStr;
use serde_json::json;

#[post("/")]
async fn post_event(req: HttpRequest, payload: web::Payload) -> Result<String, actix_web::Error> {
    let event = cloudevents_sdk_actix_web::request_to_event(&req, payload).await?;
    println!("Received Event: {:?}", event);
    Ok(format!("{:?}", event))
}

#[get("/")]
async fn get_event() -> Result<HttpResponse, actix_web::Error> {
    let payload = json!({"hello": "world"});

    Ok(cloudevents_sdk_actix_web::event_to_response(
        EventBuilder::new()
            .id("0001")
            .ty("example.test")
            .source(Url::from_str("http://localhost/").unwrap())
            .data("application/json", payload)
            .extension("someint", "10")
            .build(),
        HttpResponse::Ok()
    ).await?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(post_event)
            .service(get_event)
    })
        .bind("127.0.0.1:8080")?
        .workers(1)
        .run()
        .await
}
