//! Provides protocol binding implementations for [`crate::Event`].

#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "axum")]
pub mod axum;
#[cfg(any(
    feature = "http-binding",
    feature = "actix",
    feature = "warp",
    feature = "reqwest",
    feature = "axum"
))]
pub mod http;
#[cfg(feature = "rdkafka")]
pub mod rdkafka;
#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "warp")]
pub mod warp;

#[cfg(feature = "rdkafka")]
pub(crate) mod kafka {
    pub static SPEC_VERSION_HEADER: &str = "ce_specversion";
    pub fn header_prefix(name: &str) -> String {
        super::header_prefix("ce_", name)
    }
}

pub(crate) static CLOUDEVENTS_JSON_HEADER: &str = "application/cloudevents+json";
pub(crate) static CONTENT_TYPE: &str = "content-type";

fn header_prefix(prefix: &str, name: &str) -> String {
    if name == "datacontenttype" {
        CONTENT_TYPE.to_string()
    } else {
        [prefix, name].concat()
    }
}

#[macro_export]
macro_rules! header_value_to_str {
    ($header_value:expr) => {
        $header_value
            .to_str()
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    };
}
