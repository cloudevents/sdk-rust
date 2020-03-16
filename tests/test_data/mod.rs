use cloudevents::{Event, EventBuilder};
use serde_json::{Value, json};
use chrono::{Utc, DateTime, TimeZone};

pub fn id() -> String {
    "0001".to_string()
}

pub fn ty() -> String {
    "test_event.test_application".to_string()
}

pub fn source() -> String {
    "http://localhost".to_string()
}

pub fn datacontenttype() -> String {
    "application/json".to_string()
}

pub fn dataschema() -> String {
    "http://localhost/schema".to_string()
}

pub fn data() -> Value {
    json!({"hello": "world"})
}

pub fn data_base_64() -> Vec<u8> {
    serde_json::to_vec(&json!({"hello": "world"})).unwrap()
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

pub fn minimal_v1() -> Event {
    EventBuilder::v10()
        .id(id())
        .source(source())
        .ty(ty())
        .build()
}

pub fn minimal_v1_json() -> Value {
    json!({
        "specversion": "1.0",
        "id": id(),
        "type": ty(),
        "source": source(),
    })
}

pub fn full_v1_no_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .build()
}

pub fn full_v1_no_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "1.0",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value
    })
}

pub fn full_v1_json_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data_with_schema(datacontenttype(), dataschema(), data())
        .build()
}

pub fn full_v1_json_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "1.0",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": datacontenttype(),
        "dataschema": dataschema(),
        "data": data()
    })
}

pub fn full_v1_binary_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data_with_schema(datacontenttype(), dataschema(), data_base_64())
        .build()
}

pub fn full_v1_base64_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    let d = base64::encode(&data_base_64());

    json!({
        "specversion": "1.0",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": datacontenttype(),
        "dataschema": dataschema(),
        "data_base64": d
    })
}
