use cloudevents::{Event, EventBuilder, EventBuilderV10};
use poem::error::InternalServerError;
use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::{get, handler, Endpoint, EndpointExt, Response, Result, Route, Server};
use serde_json::json;

#[handler]
async fn get_event() -> Result<Event> {
    let event = EventBuilderV10::new()
        .id("1")
        .source("url://example_response/")
        .ty("example.ce")
        .data(
            "application/json",
            json!({
                "name": "John Doe",
                "age": 43,
                "phones": [
                    "+44 1234567",
                    "+44 2345678"
                ]
            }),
        )
        .build()
        .map_err(InternalServerError)?;
    Ok(event)
}

#[handler]
async fn post_event(event: Event) -> Event {
    tracing::debug!("received cloudevent {}", &event);
    event
}

fn echo_app() -> impl Endpoint<Output = Response> {
    Route::new()
        .at("/", get(get_event).post(post_event))
        .with(Tracing)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "poem=debug")
    }
    tracing_subscriber::fmt::init();

    let server = Server::new(TcpListener::bind("127.0.0.1:8080"));
    server.run(echo_app()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use poem::http::Method;
    use poem::{Body, Request};
    use serde_json::json;

    #[tokio::test]
    async fn poem_test() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "poem_example=debug")
        }
        tracing_subscriber::fmt::init();

        let app = echo_app();
        let time = Utc::now();
        let j = json!({"hello": "world"});
        let request = Request::builder()
            .method(Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", time.to_rfc3339())
            .header("content-type", "application/json")
            .body(Body::from_json(&j).unwrap());

        let resp: Response = app.call(request).await.unwrap();
        assert_eq!(
            resp.headers()
                .get("ce-specversion")
                .unwrap()
                .to_str()
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            resp.headers().get("ce-id").unwrap().to_str().unwrap(),
            "0001"
        );
        assert_eq!(
            resp.headers().get("ce-type").unwrap().to_str().unwrap(),
            "example.test"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost/"
        );
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
        assert_eq!(
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );

        assert_eq!(
            j.to_string().as_bytes(),
            resp.into_body().into_vec().await.unwrap()
        );
    }
}
