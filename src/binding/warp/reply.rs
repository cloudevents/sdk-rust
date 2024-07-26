use crate::{AttributesReader, Data, Event};

use warp::{http::StatusCode, reply::Reply, reply::Response};
use warp_lib as warp;

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
    let mut builder =
        warp::http::response::Response::builder().status(StatusCode::OK);

    if let Some(dct) = event.datacontenttype() {
        builder = builder.header("Content-Type", dct);
    }

    for (key, value) in event.iter() {
        builder =
            builder.header(format!("ce-{key}").as_str(), value.to_string());
    }

    let body = match event.data {
        Some(data) => match data {
            Data::Binary(v) => hyper::Body::from(v),
            Data::String(s) => hyper::Body::from(s),
            Data::Json(j) => match serde_json::to_vec(&j) {
                Ok(s) => hyper::Body::from(s),
                Err(e) => {
                    builder = builder.status(StatusCode::INTERNAL_SERVER_ERROR);
                    hyper::Body::from(e.to_string())
                }
            },
        },
        None => hyper::Body::empty(),
    };

    builder.body(body).into_response()
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;

    use http;
    use http_body_util::BodyExt;
    use warp_lib as warp;

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
