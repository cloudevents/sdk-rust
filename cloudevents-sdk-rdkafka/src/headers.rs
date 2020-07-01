use cloudevents::event::SpecVersion;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::collections::HashMap;

macro_rules! unwrap_optional_header {
    ($headers:expr, $index:expr) => {
        $headers
            .get($index)
            .unwrap()
            .map(|a| header_value_to_str!(a.1))
    };
}

macro_rules! header_value_to_str {
    ($header_value:expr) => {
        $header_value
            .to_str()
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })
    };
}

/*
macro_rules! str_name_to_header {
    ($attribute:expr,$value: expr) => {
        rdkafka::message::OwnedHeaders.add(name: &$attribute, value: &expr).map_err(|e| {
            cloudevents::message::Error::Other {
                source: Box::new(e),
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
            (s, reqwest::header::CONTENT_TYPE)
        } else {
            (s, attribute_name_to_header!(s).unwrap())
        }
    })
    .collect()
}
*/