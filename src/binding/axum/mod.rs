//! This module integrates the [cloudevents-sdk](https://docs.rs/cloudevents-sdk) with [Axum web service framework](https://docs.rs/axum/)
//! to easily send and receive CloudEvents.
//!
//! To deserialize an HTTP request as CloudEvent
//!
//! To echo events:
//!
//! ```
//! use axum_lib as axum;
//! use axum::{
//!     handler::{get, post},
//!     routing::BoxRoute,
//!     Router,
//! };
//! use cloudevents::Event;
//! use http::StatusCode;
//!
//! fn app() -> Router<BoxRoute> {
//!     Router::new()
//!         .route("/", get(|| async { "hello from cloudevents server" }))
//!         .route(
//!             "/",
//!             post(|event: Event| async move {
//!                 println!("received cloudevent {}", &event);
//!                 (StatusCode::OK, event)
//!             }),
//!         )
//!         .boxed()
//! }
//!
//! ```
//!
//! To create event inside request handlers and send them as responses:
//!
//! ```
//! use axum_lib as axum;
//! use axum::{
//!     handler::{get, post},
//!     routing::BoxRoute,
//!     Router,
//! };
//! use cloudevents::{Event, EventBuilder, EventBuilderV10};
//! use http::StatusCode;
//! use serde_json::json;
//!
//! fn app() -> Router<BoxRoute> {
//!     Router::new()
//!         .route("/", get(|| async { "hello from cloudevents server" }))
//!         .route(
//!             "/",
//!             post(|| async move {
//!                 let event = EventBuilderV10::new()
//!                     .id("1")
//!                     .source("url://example_response/")
//!                     .ty("example.ce")
//!                     .data(
//!                         mime::APPLICATION_JSON.to_string(),
//!                         json!({
//!                             "name": "John Doe",
//!                             "age": 43,
//!                             "phones": [
//!                                 "+44 1234567",
//!                                 "+44 2345678"
//!                             ]
//!                         }),
//!                     )
//!                     .build()
//!                     .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
//!
//!                 Ok::<Event, (StatusCode, String)>(event)
//!             }),
//!         )
//!         .boxed()
//! }
//!
//! ```

pub mod extract;
pub mod response;

#[cfg(test)]
mod tests {

    use axum_lib as axum;

    use axum::{
        body::Body,
        handler::{get, post},
        http::{self, Request, StatusCode},
        routing::BoxRoute,
        Router,
    };
    use chrono::Utc;
    use serde_json::json;
    use tower::ServiceExt; // for `app.oneshot()`

    use crate::Event;

    fn echo_app() -> Router<BoxRoute> {
        Router::new()
            .route("/", get(|| async { "hello from cloudevents server" }))
            .route(
                "/",
                post(|event: Event| async move {
                    println!("received cloudevent {}", &event);
                    (StatusCode::OK, event)
                }),
            )
            .boxed()
    }

    #[tokio::test]
    async fn axum_mod_test() {
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
