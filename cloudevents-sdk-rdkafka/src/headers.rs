use cloudevents::event::SpecVersion;
use lazy_static::lazy_static;
use std::collections::HashMap;



/*
macro_rules! unwrap_optional_header {
    ($headers:expr, $index:expr) => {
        $headers
            .get($index)
            .unwrap()
            .map(|a| header_value_to_str!(a.1))
    };
}
*/

macro_rules! header_value_to_str {
    ($header_value:expr) => {
        str::from_utf8($header_value)
            .map_err(|e| cloudevents::message::Error::Other {
                source: Box::new(e),
            })
    };
}

macro_rules! attribute_name_to_header {
    ($attribute:expr) => {
        format!("{}{}","ce_", $attribute )
    };
} 

fn attributes_to_headers(
    it: impl Iterator<Item = &'static str>,
) -> HashMap<&'static str, String> {
    it.map(|s| {
        if s == "datacontenttype" {
            (s, String::from("content-type"))
        } else {
            (s, attribute_name_to_header!(s))
        }
    })
    .collect()
}

lazy_static! {
    pub(crate) static ref ATTRIBUTES_TO_HEADERS: HashMap<&'static str, String> =
        attributes_to_headers(SpecVersion::all_attribute_names());
    
    pub(crate) static ref SPEC_VERSION_HEADER: &'static str = "ce_specversion";

    pub(crate) static ref CLOUDEVENTS_JSON_HEADER: &'static str = "application/cloudevents+json";
}

