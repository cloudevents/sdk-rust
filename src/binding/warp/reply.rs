use warp_lib as warp;

use crate::binding::http::builder::adapter::to_response;

use crate::Event;
use http::StatusCode;
use warp::reply::Response;

///
/// # Serializes [`crate::Event`] as a http response
///
/// ```
/// # use warp_lib as warp;
/// use cloudevents::binding::warp::reply::from_event;
/// use cloudevents::Event;
/// use warp::Filter;
/// use warp::Reply;
///
/// let routes = warp::any()
///    .map(|| from_event(Event::default()));
/// ```
pub fn from_event(event: Event) -> Response {
    match to_response(event) {
        Ok(response) => response,
        Err(e) => warp::http::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(hyper::body::Body::from(e.to_string()))
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;

    #[test]
    fn test_response() {
        let input = fixtures::v10::minimal_string_extension();

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
    async fn test_response_with_full_data() {
        let input = fixtures::v10::full_binary_json_data_string_extension();

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
