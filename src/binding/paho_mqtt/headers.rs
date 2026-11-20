use crate::event::SpecVersion;
use std::collections::HashMap;
use std::fmt::format;

macro_rules! attribute_name_to_header {
    ($attribute:expr) => {
        format!("ce_{}", $attribute)
    };
}

pub fn attributes_to_headers(
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

pub(crate) static SPEC_VERSION_HEADER: &'static str = "ce-specversion";
pub(crate) static CLOUDEVENTS_JSON_HEADER: &'static str = "application/cloudevents+json";
pub(crate) static CONTENT_TYPE: &'static str = "content-type";

pub enum MqttVersion {
    MQTT_3_1_1,
    MQTT_5,
}
