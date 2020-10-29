use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use cloudevents::{EventBuilder, EventBuilderV10};
use cloudevents_sdk_actix_web::{HttpResponseBuilderExt, HttpRequestExt};
use serde_json::json;

#[post("/")]
async fn post_event(req: HttpRequest, payload: web::Payload) -> Result<String, actix_web::Error> {
    let event = req.to_event(payload).await?;
    println!("Received Event: {:?}", event);
    Ok(format!("{:?}", event))
}

#[get("/")]
async fn get_event() -> Result<HttpResponse, actix_web::Error> {
    let payload = json!({"hello": "world"});

    Ok(HttpResponse::Ok()
        .event(
            EventBuilderV10::new()
                .id("0001")
                .ty("example.test")
                .source("http://localhost/")
                .data("application/json", payload)
                .extension("someint", "10")
                .build()
                .unwrap(),
        )
        .await?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .service(post_event)
            .service(get_event)
    })
    .bind("127.0.0.1:9000")?
    .workers(1)
    .run()
    .await
}
