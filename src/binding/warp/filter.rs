use warp_lib as warp;

use crate::binding::http;

use crate::Event;
use warp::http::HeaderMap;
use warp::Filter;
use warp::Rejection;

#[derive(Debug)]
#[allow(dead_code)]
pub struct EventFilterError {
    error: crate::message::Error,
}

impl warp::reject::Reject for EventFilterError {}

///
/// # Extracts [`crate::Event`] from incoming request
///
/// ```
/// # use warp_lib as warp;
/// use cloudevents::binding::warp::filter::to_event;
/// use warp::Filter;
/// use warp::Reply;
///
/// let routes = warp::any()
///    .and(to_event())
///    .map(|event| {
///         // do something with the event
///     }
/// );
/// ```
///
pub fn to_event() -> impl Filter<Extract = (Event,), Error = Rejection> + Copy {
    warp::header::headers_cloned()
        .and(warp::body::bytes())
        .and_then(create_event)
}

async fn create_event(headers: HeaderMap, body: bytes::Bytes) -> Result<Event, Rejection> {
    http::to_event(&headers, body.to_vec())
        .map_err(|error| warp::reject::custom(EventFilterError { error }))
}

#[cfg(test)]
mod tests {
    use super::to_event;
    use crate::test::fixtures;
    use std::convert::TryInto;
    use warp::test;
    use warp_lib as warp;

    #[tokio::test]
    async fn test_request() {
        let expected = fixtures::v10::minimal_string_extension();

        let result = test::request()
            .method("POST")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "test_event.test_application")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .filter(&to_event())
            .await
            .unwrap();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn test_bad_request() {
        let result = test::request()
            .method("POST")
            .header("ce-specversion", "BAD SPECIFICATION")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", fixtures::time().to_rfc3339())
            .filter(&to_event())
            .await;

        assert!(result.is_err());
        let rejection = result.unwrap_err();

        let reason = rejection.find::<super::EventFilterError>().unwrap();
        assert_eq!(
            reason.error.to_string(),
            "Invalid specversion BAD SPECIFICATION"
        )
    }

    #[tokio::test]
    async fn test_request_with_full_data() {
        let expected = fixtures::v10::full_binary_json_data_string_extension();

        let result = test::request()
            .method("POST")
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
            .json(&fixtures::json_data())
            .filter(&to_event())
            .await
            .unwrap();

        let mut event = result.clone();
        let (_datacontenttype, _dataschema, data) = event.take_data();
        let actual_payload: Vec<u8> = data.unwrap().try_into().unwrap();
        let expected_payload: Vec<u8> = serde_json::to_vec(&fixtures::json_data()).unwrap();
        assert_eq!(expected_payload, actual_payload);

        assert_eq!(expected, result);
    }
}
