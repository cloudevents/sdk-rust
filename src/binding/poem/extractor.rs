use crate::binding::http::to_event;
use crate::Event;

use poem_lib::error::ResponseError;
use poem_lib::http::StatusCode;
use poem_lib::{FromRequest, Request, RequestBody, Result};

impl ResponseError for crate::message::Error {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl<'a> FromRequest<'a> for Event {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        Ok(to_event(req.headers(), body.take()?.into_vec().await?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::fixtures;
    use poem_lib::http::Method;

    #[tokio::test]
    async fn test_request() {
        let expected = fixtures::v10::minimal_string_extension();

        let req = Request::builder()
            .method(Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .finish();
        let (req, mut body) = req.split();
        let result = Event::from_request(&req, &mut body).await.unwrap();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn test_bad_request() {
        let req = Request::builder()
            .method(Method::POST)
            .header("ce-specversion", "BAD SPECIFICATION")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", fixtures::time().to_rfc3339())
            .finish();

        let (req, mut body) = req.split();
        let resp = Event::from_request(&req, &mut body).await.err().unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(resp.to_string(), "Invalid specversion BAD SPECIFICATION");
    }

    #[tokio::test]
    async fn test_request_with_full_data() {
        let expected = fixtures::v10::full_binary_json_data_string_extension();

        let req = Request::builder()
            .method(Method::POST)
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-subject", "cloudevents-sdk")
            .header("content-type", "application/json")
            .header("ce-string_ex", "val")
            .header("ce-int_ex", "10")
            .header("ce-bool_ex", "true")
            .header("ce-time", fixtures::time().to_rfc3339())
            .body(fixtures::json_data_binary());
        let (req, mut body) = req.split();
        let result = Event::from_request(&req, &mut body).await.unwrap();

        assert_eq!(expected, result);
    }
}
