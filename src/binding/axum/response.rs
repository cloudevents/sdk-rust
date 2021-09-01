use axum_lib as axum;

use axum::{body::Body, http::Response, response::IntoResponse};
use http::{header, StatusCode};

use super::server_response::event_to_response;
use crate::event::Event;

impl IntoResponse for Event {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Body> {
        match event_to_response(self) {
            Ok(resp) => resp,
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(err.to_string().into())
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::fixtures;

    #[test]
    fn axum_test_response() {
        let input = fixtures::v10::minimal_string_extension();

        let resp = input.into_response();

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
            "test_event.test_application"
        );
        assert_eq!(
            resp.headers().get("ce-source").unwrap().to_str().unwrap(),
            "http://localhost/"
        );
        assert_eq!(
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );
    }

    #[tokio::test]
    async fn axum_test_response_with_full_data() {
        let input = fixtures::v10::full_binary_json_data_string_extension();

        let resp = input.into_response();

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
            "test_event.test_application"
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
            resp.headers().get("ce-int_ex").unwrap().to_str().unwrap(),
            "10"
        );

        let (_, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await.unwrap();

        assert_eq!(fixtures::json_data_binary(), body);
    }
}
