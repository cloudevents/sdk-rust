use crate::binding::http::{Builder, Serializer};
use crate::message::{BinaryDeserializer, Result};
use crate::Event;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder};

impl Builder<HttpResponse> for HttpResponseBuilder {
    fn header(&mut self, key: &str, value: http::header::HeaderValue) {
        self.insert_header((key, value));
    }
    fn body(&mut self, bytes: Vec<u8>) -> Result<HttpResponse> {
        Ok(HttpResponseBuilder::body(self, bytes))
    }
    fn finish(&mut self) -> Result<HttpResponse> {
        Ok(HttpResponseBuilder::finish(self))
    }
}

/// Method to fill an [`HttpResponseBuilder`] with an [`Event`].
pub fn event_to_response<T: Builder<HttpResponse> + 'static>(
    event: Event,
    response: T,
) -> std::result::Result<HttpResponse, actix_web::error::Error> {
    BinaryDeserializer::deserialize_binary(event, Serializer::new(response))
        .map_err(actix_web::error::ErrorBadRequest)
}

/// So that an actix-web handler may return an Event
impl actix_web::Responder for Event {
    fn respond_to(self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::build(StatusCode::OK).event(self).unwrap()
    }
}

/// Extension Trait for [`HttpResponseBuilder`] which acts as a wrapper for the function [`event_to_response()`].
///
/// This trait is sealed and cannot be implemented for types outside of this crate.
pub trait HttpResponseBuilderExt: private::Sealed {
    /// Fill this [`HttpResponseBuilder`] with an [`Event`].
    fn event(self, event: Event) -> std::result::Result<HttpResponse, actix_web::Error>;
}

impl HttpResponseBuilderExt for HttpResponseBuilder {
    fn event(self, event: Event) -> std::result::Result<HttpResponse, actix_web::Error> {
        event_to_response(event, self)
    }
}

// Sealing the HttpResponseBuilderExt
mod private {
    pub trait Sealed {}
    impl Sealed for actix_web::HttpResponseBuilder {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test::fixtures;
    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_response() {
        let input = fixtures::v10::minimal_string_extension();

        let resp = HttpResponseBuilder::new(StatusCode::OK)
            .event(input)
            .unwrap();

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

    #[actix_rt::test]
    async fn test_response_with_full_data() {
        let input = fixtures::v10::full_binary_json_data_string_extension();

        let resp = HttpResponseBuilder::new(StatusCode::OK)
            .event(input)
            .unwrap();

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

        // let bytes = test::load_stream(resp.take_body().into_stream())
        //     .await
        //     .unwrap();
        let bytes = test::load_body(resp.into_body()).await.unwrap();
        assert_eq!(fixtures::json_data_binary(), bytes.as_ref())
    }
}
