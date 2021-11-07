use async_trait::async_trait;
use poem_lib::error::ReadBodyError;
use poem_lib::http::StatusCode;
use poem_lib::{FromRequest, IntoResponse, Request, RequestBody, Response};

use crate::binding::http::to_event;
use crate::Event;

#[derive(Debug)]
pub enum ParseEventError {
    ReadBody(ReadBodyError),
    ParseEvent(crate::message::Error),
}

impl From<ReadBodyError> for ParseEventError {
    fn from(err: ReadBodyError) -> Self {
        ParseEventError::ReadBody(err)
    }
}

impl From<crate::message::Error> for ParseEventError {
    fn from(err: crate::message::Error) -> Self {
        ParseEventError::ParseEvent(err)
    }
}

impl IntoResponse for ParseEventError {
    fn into_response(self) -> Response {
        match self {
            ParseEventError::ReadBody(err) => err.into_response(),
            ParseEventError::ParseEvent(err) => (StatusCode::BAD_REQUEST, err.to_string()).into(),
        }
    }
}

#[async_trait]
impl<'a> FromRequest<'a> for Event {
    type Error = ParseEventError;

    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self, Self::Error> {
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
        let resp = Event::from_request(&req, &mut body).await.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.into_body().into_string().await.unwrap(),
            "Invalid specversion BAD SPECIFICATION"
        );
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
            .header("ce-time", &fixtures::time().to_rfc3339())
            .body(fixtures::json_data_binary());
        let (req, mut body) = req.split();
        let result = Event::from_request(&req, &mut body).await.unwrap();

        assert_eq!(expected, result);
    }
}
