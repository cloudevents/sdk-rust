use axum::{
    routing::{get, post},
    Router,
};
use cloudevents::Event;
use http::StatusCode;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

fn echo_app() -> Router {
    Router::new()
        .route("/", get(|| async { "hello from cloudevents server" }))
        .route(
            "/",
            post(|event: Event| async move {
                tracing::debug!("received cloudevent {}", &event);
                (StatusCode::OK, event)
            }),
        )
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "axum_example=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();
    let service = echo_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(service.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {

    use super::echo_app;

    use axum::{
        body::Body,
        http::{self, Request},
    };
    use chrono::Utc;
    use hyper;
    use serde_json::json;
    use tower::ServiceExt; // for `app.oneshot()`

    #[tokio::test]
    async fn axum_mod_test() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "axum_example=debug,tower_http=debug")
        }
        tracing_subscriber::fmt::init();

        let app = echo_app();
        let time = Utc::now();
        let j = json!({"hello": "world"});
        let request = Request::builder()
            .method(http::Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", time.to_rfc3339())
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&j).unwrap()))
            .unwrap();

        let resp = app.oneshot(request).await.unwrap();
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

        let (_, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await.unwrap();

        assert_eq!(j.to_string().as_bytes(), body);
    }
}
