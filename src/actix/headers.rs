use crate::event::SpecVersion;
use actix_web::http::header;
use actix_web::http::{HeaderName, HeaderValue};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::str::FromStr;

macro_rules! header_value_to_str {
    ($header_value:expr) => {
        $header_value
            .to_str()
            .map_err(|e| crate::message::Error::Other {
                source: Box::new(e),
            })
    };
}

macro_rules! str_to_header_value {
    ($header_value:expr) => {
        HeaderValue::from_str($header_value).map_err(|e| crate::message::Error::Other {
            source: Box::new(e),
        })
    };
}

macro_rules! str_name_to_header {
    ($attribute:expr) => {
        HeaderName::from_str($attribute).map_err(|e| crate::message::Error::Other {
            source: Box::new(e),
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
            (s, header::CONTENT_TYPE)
        } else {
            (s, attribute_name_to_header!(s).unwrap())
        }
    })
    .collect()
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_HEADERS: HashMap<&'static str, HeaderName> =
        attributes_to_headers(SpecVersion::all_attribute_names());
    pub(crate) static ref CLOUDEVENTS_JSON_HEADER: HeaderValue =
        HeaderValue::from_static("application/cloudevents+json");
}

pub(crate) static SPEC_VERSION_HEADER: &str = "ce-specversion";
