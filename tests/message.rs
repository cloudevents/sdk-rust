mod test_data;
use test_data::*;
use cloudevents::Event;
use cloudevents::message::{BinaryDeserializer, StructuredDeserializer, BinaryVisitor, MessageAttributeValue, Error, SerializationResult};

#[test]
fn message_v03_roundtrip_structured() {
    let mut out_event = Event::default();
    v03::full_json_data().deserialize_structured(&mut out_event);
    assert_eq!(v03::full_json_data(), out_event)
}

#[test]
fn message_v03_roundtrip_binary() {
    let mut out_event = Event::default();
    v03::full_json_data().deserialize_binary(&mut out_event);
    assert_eq!(v03::full_json_data(), out_event)
}

#[test]
fn message_v10_roundtrip_structured() {
    let mut out_event = Event::default();
    v10::full_json_data().deserialize_structured(&mut out_event);
    assert_eq!(v10::full_json_data(), out_event)
}

#[test]
fn message_v10_roundtrip_binary() {
    let mut out_event = Event::default();
    v10::full_json_data().deserialize_binary(&mut out_event);
    assert_eq!(v10::full_json_data(), out_event)
}

#[test]
fn message_v03_invalid_attribute_name() {
    assert_eq!(
        Error::UnrecognizedAttributeName {name: "dataschema".to_string()}.to_string(),
        v03::full_json_data().set_attribute("dataschema", MessageAttributeValue::Boolean(true)).unwrap_err().to_string()
    )
}

#[test]
fn message_v10_invalid_attribute_name() {
    assert_eq!(
        Error::UnrecognizedAttributeName {name: "schemaurl".to_string()}.to_string(),
        v10::full_json_data().set_attribute("schemaurl", MessageAttributeValue::Boolean(true)).unwrap_err().to_string()
    )
}