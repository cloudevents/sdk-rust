use cloudevents::event::SpecVersion;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;
use surf::http::headers::{HeaderName, HeaderValue};

macro_rules! header_to_str {
    ($header_value:expr) => {
        $header_value.unwrap().as_str()
    };
}

macro_rules! str_name_to_header {
    ($attribute:expr) => {
        surf::http::headers::HeaderName::from_str($attribute).map_err(|e| {
            cloudevents::message::Error::Other {
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
        })
    };
}

macro_rules! attribute_name_to_header {
    ($attribute:expr) => {
        str_name_to_header!(&["ce-", $attribute].concat())
    };
}

fn attributes_to_headers(
    it: impl Iterator<Item = &'static str>,
) -> HashMap<&'static str, HeaderName> {
    it.map(|s| {
        if s == "datacontenttype" {
            (s, surf::http::headers::CONTENT_TYPE)
        } else {
            (s, attribute_name_to_header!(s).unwrap())
        }
    })
    .collect()
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_HEADERS: HashMap<&'static str, HeaderName> =
        attributes_to_headers(SpecVersion::all_attribute_names());
    pub(crate) static ref SPEC_VERSION_HEADER: HeaderName =
        HeaderName::from_str("ce-specversion").unwrap();
    pub(crate) static ref CLOUDEVENTS_JSON_HEADER: HeaderValue =
        HeaderValue::from_str("application/cloudevents+json").unwrap();
}
