use super::*;
use cloudevents::{Event, EventBuilder};
use serde_json::{json, Value};
use url::Url;

pub fn minimal() -> Event {
    EventBuilder::v10()
        .id(id())
        .source(Url::parse(source().as_ref()).unwrap())
        .ty(ty())
        .build()
}

pub fn minimal_json() -> Value {
    json!({
        "specversion": "1.0",
        "id": id(),
        "type": ty(),
        "source": source(),
    })
}

pub fn full_no_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(Url::parse(source().as_ref()).unwrap())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .build()
}

pub fn full_no_data_json() -> Value {
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

pub fn full_json_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(Url::parse(source().as_ref()).unwrap())
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
}

pub fn full_json_data_json() -> Value {
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
        "datacontenttype": json_datacontenttype(),
        "dataschema": dataschema(),
        "data": json_data()
    })
}

pub fn full_json_base64_data_json() -> Value {
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
        "datacontenttype": json_datacontenttype(),
        "dataschema": dataschema(),
        "data_base64": base64::encode(&json_data_binary())
    })
}

pub fn full_xml_string_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(Url::parse(source().as_ref()).unwrap())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data(xml_datacontenttype(), xml_data())
        .build()
}

pub fn full_xml_binary_data() -> Event {
    let (string_ext_name, string_ext_value) = string_extension();
    let (bool_ext_name, bool_ext_value) = bool_extension();
    let (int_ext_name, int_ext_value) = int_extension();

    EventBuilder::v10()
        .id(id())
        .source(Url::parse(source().as_ref()).unwrap())
        .ty(ty())
        .subject(subject())
        .time(time())
        .extension(&string_ext_name, string_ext_value)
        .extension(&bool_ext_name, bool_ext_value)
        .extension(&int_ext_name, int_ext_value)
        .data(xml_datacontenttype(), Vec::from(xml_data()))
        .build()
}

pub fn full_xml_string_data_json() -> Value {
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
        "datacontenttype": xml_datacontenttype(),
        "data": xml_data()
    })
}

pub fn full_xml_base64_data_json() -> Value {
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
        "datacontenttype": xml_datacontenttype(),
        "data_base64": base64::encode(Vec::from(xml_data()))
    })
}
