use crate::server_request::request_to_event;

use cloudevents::Event;
use warp::http::HeaderMap;
use warp::Filter;
use warp::Rejection;

#[derive(Debug)]
pub struct EventFilterError {
    error: cloudevents::message::Error,
}

impl warp::reject::Reject for EventFilterError {}

///
/// # Extracts [`cloudevents::Event`] from incoming request
///
/// ```
/// use cloudevents_sdk_warp::filter::to_event;
/// use warp::Filter;
/// use warp::Reply;
///
/// let routes = warp::any()
///    .and(to_event())
///    .map(|event| {
///         // do something with the event
///     }
///     );
/// ```
///
pub fn to_event() -> impl Filter<Extract = (Event,), Error = Rejection> + Copy {
    warp::header::headers_cloned()
        .and(warp::body::bytes())
        .and_then(create_event)
}

async fn create_event(headers: HeaderMap, body: bytes::Bytes) -> Result<Event, Rejection> {
    request_to_event(headers, body)
        .map_err(|error| warp::reject::custom(EventFilterError { error }))
}

#[cfg(test)]
mod tests {
    use super::to_event;
    use warp::test;

    use chrono::Utc;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;

    #[tokio::test]
    async fn test_request() {
        let time = Utc::now();
        let expected = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .time(time)
            .extension("someint", "10")
            .build()
            .unwrap();

        let result = test::request()
            .method("POST")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", time.to_rfc3339())
            .filter(&to_event())
            .await
            .unwrap();

        assert_eq!(expected, result);
    }

    #[tokio::test]
    async fn test_bad_request() {
        let time = Utc::now();

        let result = test::request()
            .method("POST")
            .header("ce-specversion", "BAD SPECIFICATION")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost/")
            .header("ce-someint", "10")
            .header("ce-time", time.to_rfc3339())
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

        let result = test::request()
            .method("POST")
            .header("ce-specversion", "1.0")
            .header("ce-id", "0001")
            .header("ce-type", "example.test")
            .header("ce-source", "http://localhost")
            .header("ce-someint", "10")
            .header("ce-time", time.to_rfc3339())
            .header("content-type", "application/json")
            .json(&j)
            .filter(&to_event())
            .await
            .unwrap();

        assert_eq!(expected, result);
    }
}
