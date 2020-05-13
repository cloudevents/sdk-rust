mod test_data;
use cloudevents::message::{
    BinaryDeserializer, BinarySerializer, DeserializationResult, Error, MessageAttributeValue,
    MessageDeserializer, SerializationResult, StructuredDeserializer,
};
use cloudevents::Event;
use test_data::*;

#[test]
fn message_v03_roundtrip_structured() -> DeserializationResult {
    assert_eq!(
        v03::full_json_data(),
        StructuredDeserializer::into_event(v03::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v03_roundtrip_binary() -> DeserializationResult {
    assert_eq!(
        v03::full_json_data(),
        BinaryDeserializer::into_event(v03::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v10_roundtrip_structured() -> DeserializationResult {
    assert_eq!(
        v10::full_json_data(),
        StructuredDeserializer::into_event(v10::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v10_roundtrip_binary() -> DeserializationResult {
    assert_eq!(
        v10::full_json_data(),
        BinaryDeserializer::into_event(v10::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v03_invalid_attribute_name() {
    assert_eq!(
        Error::UnrecognizedAttributeName {
            name: "dataschema".to_string()
        }
        .to_string(),
        v03::full_json_data()
            .set_attribute("dataschema", MessageAttributeValue::Boolean(true))
            .unwrap_err()
            .to_string()
    )
}

#[test]
fn message_v10_invalid_attribute_name() {
    assert_eq!(
        Error::UnrecognizedAttributeName {
            name: "schemaurl".to_string()
        }
        .to_string(),
        v10::full_json_data()
            .set_attribute("schemaurl", MessageAttributeValue::Boolean(true))
            .unwrap_err()
            .to_string()
    )
}
