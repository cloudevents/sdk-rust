use actix_web::{get, post, App, HttpServer};
use cloudevents::{Event, EventBuilder, EventBuilderV10, test_data::*};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_post() {
        use actix_web::{test, App};
        use cloudevents::AttributesReader;

        let expected = v10::minimal();

        let req = test::TestRequest::post()
        .header("ce-specversion", "1.0")
        .header("ce-id", "0001")
        .header("ce-type", "test_event.test_application")
        .header("ce-source", "http://localhost/")
        .to_request();

        let mut app = test::init_service(App::new().service(post_event)).await;
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            expected.id()
        );
    }

}
