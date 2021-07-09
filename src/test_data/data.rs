use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};

pub fn id() -> String {
    "0001".to_string()
}

pub fn ty() -> String {
    "test_event.test_application".to_string()
}

pub fn source() -> String {
    "http://localhost/".to_string()
}

pub fn json_datacontenttype() -> String {
    "application/json".to_string()
}

pub fn xml_datacontenttype() -> String {
    "application/xml".to_string()
}

pub fn dataschema() -> String {
    "http://localhost/schema".to_string()
}

pub fn json_data() -> Value {
    json!({"hello": "world"})
}

pub fn json_data_binary() -> Vec<u8> {
    serde_json::to_vec(&json!({"hello": "world"})).unwrap()
}

pub fn xml_data() -> String {
    "<hello>world</hello>".to_string()
}

pub fn subject() -> String {
    "cloudevents-sdk".to_string()
}

pub fn time() -> DateTime<Utc> {
    Utc.ymd(2020, 3, 16).and_hms(11, 50, 00)
}

pub fn string_extension() -> (String, String) {
    ("string_ex".to_string(), "val".to_string())
}

pub fn bool_extension() -> (String, bool) {
    ("bool_ex".to_string(), true)
}

pub fn int_extension() -> (String, i64) {
    ("int_ex".to_string(), 10)
}
