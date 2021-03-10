use crate::server_response::event_to_response;

use cloudevents::Event;
use http::StatusCode;
use warp::reply::Response;

///
/// # Serializes [`cloudevents::Event`] as a http response
///
/// ```
/// use cloudevents_sdk_warp::reply::from_event;
/// use cloudevents::Event;
/// use warp::Filter;
/// use warp::Reply;
///
/// let routes = warp::any()
///    .map(|| from_event(Event::default()));
/// ```
pub fn from_event(event: Event) -> Response {
    match event_to_response(event) {
        Ok(response) => response,
        Err(e) => warp::http::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(hyper::body::Body::from(e.to_string()))
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {

    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;

    #[test]
    fn test_response() {
        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost/")
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = super::from_event(input);

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
            resp.headers().get("ce-someint").unwrap().to_str().unwrap(),
            "10"
        );
    }

    #[tokio::test]
    async fn test_response_with_full_data() {
        let j = json!({"hello": "world"});

        let input = EventBuilderV10::new()
            .id("0001")
            .ty("example.test")
            .source("http://localhost")
            .data("application/json", j.clone())
            .extension("someint", "10")
            .build()
            .unwrap();

        let resp = super::from_event(input);

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
            "http://localhost"
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
