use actix_web::http::header;
use actix_web::http::{HeaderName, HeaderValue};
use cloudevents::event::SpecVersion;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;

macro_rules! unwrap_optional_header {
    ($headers:expr, $name:expr) => {
        $headers
          .get::<&'static HeaderName>(&$name)
          .map(|a| header_value_to_str!(a))
    };
}

macro_rules! header_value_to_str {
    ($header_value:expr) => {
        $header_value.to_str().map_err(|e| cloudevents::message::Error::Other{ source: Box::new(e) })
    };
}

macro_rules! str_to_header_value {
    ($header_value:expr) => {
        HeaderValue::from_str($header_value).map_err(|e| cloudevents::message::Error::Other{ source: Box::new(e) })
    };
}

macro_rules! str_name_to_header {
    ($attribute:expr) => {
        HeaderName::from_str($attribute).map_err(|e| cloudevents::message::Error::Other{ source: Box::new(e) })
    };
}

macro_rules! attribute_name_to_header {
    ($attribute:expr) => {
        str_name_to_header!(&["ce-", $attribute].concat())
    };
}

fn attributes_to_headers(
    map: &HashMap<SpecVersion, &'static [&'static str]>,
) -> HashMap<&'static str, HeaderName> {
    map.values()
        .flat_map(|s| s.iter())
        .map(|s| {
            if *s == "datacontenttype" {
                (*s, header::CONTENT_TYPE)
            } else {
                (*s, attribute_name_to_header!(s).unwrap())
            }
        })
        .collect()
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_HEADERS: HashMap<&'static str, HeaderName> =
        attributes_to_headers(&cloudevents::event::spec_version::ATTRIBUTE_NAMES);
    pub(crate) static ref SPEC_VERSION_HEADER: HeaderName =
        HeaderName::from_static("ce-specversion");
    pub(crate) static ref CLOUDEVENTS_JSON_HEADER: HeaderValue =
        HeaderValue::from_static("application/cloudevents+json");
}
