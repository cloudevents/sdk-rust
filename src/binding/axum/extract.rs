use axum_lib as axum;

use async_trait::async_trait;
use axum::extract::{FromRequest, RequestParts};
use http::StatusCode;
use http_body::Body;
use hyper::body;

use crate::binding::http::to_event;
use crate::event::Event;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl<B> FromRequest<B> for Event
where
    B: Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = req.headers().cloned().ok_or(0).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "unexpected empty headers".to_string(),
            )
        })?;

        let req_body = req
            .take_body()
            .ok_or(0)
            .map_err(|_| (StatusCode::BAD_REQUEST, "unexpected empty body".to_string()))?;

        let buf = body::to_bytes(req_body)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}", e.into())))?;
        to_event(&headers, buf.to_vec()).map_err(|e| (StatusCode::BAD_REQUEST, format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use axum_lib as axum;

    use super::*;
    use axum::body::Body;
    use axum::http::{self, Request, StatusCode};
    use chrono::Utc;
    use serde_json::json;

    use crate::{EventBuilder, EventBuilderV10};

    #[tokio::test]
    async fn axum_test_request() {
        let time = Utc::now();
        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .time(time)
            .extension("someint", "10")
            .build()
            .unwrap();

        let mut request = RequestParts::new(
            Request::builder()
                .method(http::Method::POST)
                .header("ce-specversion", "1.0")
                .header("ce-id", "0001")
                .header("ce-type", "example.test")
                .header("ce-source", "http://localhost/")
                .header("ce-someint", "10")
                .header("ce-time", time.to_rfc3339())
                .body(Body::empty())
                .unwrap(),
        );

        let result = Event::from_request(&mut request).await.unwrap();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn axum_test_bad_request() {
        let time = Utc::now();

        let mut request = RequestParts::new(
            Request::builder()
                .method(http::Method::POST)
                .header("ce-specversion", "BAD SPECIFICATION")
                .header("ce-id", "0001")
                .header("ce-type", "example.test")
                .header("ce-source", "http://localhost/")
                .header("ce-someint", "10")
                .header("ce-time", time.to_rfc3339())
                .body(Body::empty())
                .unwrap(),
        );

        let result = Event::from_request(&mut request).await;
        assert!(result.is_err());
        let rejection = result.unwrap_err();

        let reason = rejection.0;
        assert_eq!(reason, StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn axum_test_request_with_full_data() {
        let time = Utc::now();
        let j = json!({"hello": "world"});

        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .time(time)
            .data("application/json", j.to_string().into_bytes())
            .extension("someint", "10")
            .build()
            .unwrap();

        let mut request = RequestParts::new(
            Request::builder()
                .method(http::Method::POST)
                .header("ce-specversion", "1.0")
                .header("ce-id", "0001")
                .header("ce-type", "example.test")
                .header("ce-source", "http://localhost")
                .header("ce-someint", "10")
                .header("ce-time", time.to_rfc3339())
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&j).unwrap()))
                .unwrap(),
        );

        let result = Event::from_request(&mut request).await.unwrap();

        assert_eq!(expected, result);
    }
}
