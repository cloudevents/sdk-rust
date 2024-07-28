use crate::{AttributesReader, Data, Event};

use bytes::Bytes;
use poem_lib::http::StatusCode;
use poem_lib::{IntoResponse, Response};

impl IntoResponse for Event {
    fn into_response(self) -> Response {
        let mut builder = Response::builder().status(StatusCode::OK);

        if let Some(dct) = self.datacontenttype() {
            builder = builder.content_type(dct);
        }

        for (key, value) in self.iter() {
            builder = builder.header(format!("ce-{key}").as_str(), value.to_string());
        }

        match self.data {
            Some(data) => match data {
                Data::Binary(v) => builder.body(Bytes::copy_from_slice(v.as_slice())),
                Data::String(s) => builder.body(s.clone()),
                Data::Json(j) => match serde_json::to_string(&j) {
                    Ok(s) => builder.body(s),
                    Err(e) => Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(e.to_string()),
                },
            },
            None => builder.finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;
    use poem_lib::IntoResponse;

    #[test]
    fn test_response() {
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
    async fn test_response_with_full_data() {
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

        let body = resp.into_body().into_vec().await.unwrap();
        assert_eq!(fixtures::json_data_binary(), body);
    }
}
