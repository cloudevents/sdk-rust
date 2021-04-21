use super::*;
use cloudevents::{Event, EventBuilder, EventBuilderV03};
use serde_json::{json, Value};

use url::Url;

pub fn minimal() -> Event {
    EventBuilderV03::new()
        .id(id())
        .source(source())
        .ty(ty())
        .build()
        .unwrap()
}

pub fn minimal_json() -> Value {
    json!({
        "specversion": "0.3",
        "id": id(),
        "type": ty(),
        "source": source(),
    })
}

pub fn full_no_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilderV03::new()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .build()
        .unwrap()
}

pub fn full_no_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "0.3",
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

pub fn full_json_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilderV03::new()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data_with_schema(
            json_datacontenttype(),
            Url::parse(dataschema().as_ref()).unwrap(),
            json_data(),
        )
        .build()
        .unwrap()
}

pub fn full_json_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "0.3",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": json_datacontenttype(),
        "schemaurl": dataschema(),
        "data": json_data()
    })
}

pub fn full_json_base64_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "0.3",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": json_datacontenttype(),
        "schemaurl": dataschema(),
        "datacontentencoding": "base64",
        "data": base64::encode(&json_data_binary())
    })
}

pub fn full_xml_string_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilderV03::new()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data(xml_datacontenttype(), xml_data())
        .build()
        .unwrap()
}

pub fn full_xml_binary_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilderV03::new()
        .id(id())
        .source(source())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data(xml_datacontenttype(), Vec::from(xml_data()))
        .build()
        .unwrap()
}

pub fn full_xml_string_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "0.3",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": xml_datacontenttype(),
        "data": xml_data()
    })
}

pub fn full_xml_base64_data_json() -> Value {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    json!({
        "specversion": "0.3",
        "id": id(),
        "type": ty(),
        "source": source(),
        "subject": subject(),
        "time": time(),
        string_ext_name: string_ext_value,
        bool_ext_name: bool_ext_value,
        int_ext_name: int_ext_value,
        "datacontenttype": xml_datacontenttype(),
        "datacontentencoding": "base64",
        "data": base64::encode(Vec::from(xml_data()))
    })
}
