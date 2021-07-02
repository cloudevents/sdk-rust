use actix_web::{get, post, App, HttpServer};
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use serde_json::json;

#[post("/")]
async fn post_event(event: Event) -> Event {
    println!("Received Event: {:?}", event);
    event
}

#[get("/")]
async fn get_event() -> Event {
    let payload = json!({"hello": "world"});

    EventBuilderV10::new()
        .id("0001")
        .ty("example.test")
        .source("http://localhost/")
        .data("application/json", payload)
        .extension("someint", "10")
        .build()
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .wrap(actix_web::middleware::Logger::default())
            .service(post_event)
            .service(get_event)
    })
    .bind("127.0.0.1:9000")?
    .workers(1)
    .run()
    .await
}
