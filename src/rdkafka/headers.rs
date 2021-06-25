use crate::event::SpecVersion;
use lazy_static::lazy_static;
use std::collections::HashMap;

macro_rules! attribute_name_to_header {
    ($attribute:expr) => {
        format!("ce_{}", $attribute)
    };
}

fn attributes_to_headers(it: impl Iterator<Item = &'static str>) -> HashMap<&'static str, String> {
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
}

pub(crate) static SPEC_VERSION_HEADER: &str = "ce_specversion";
pub(crate) static CLOUDEVENTS_JSON_HEADER: &str = "application/cloudevents+json";
pub(crate) static CONTENT_TYPE: &str = "content-type";
