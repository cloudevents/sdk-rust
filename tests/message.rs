mod test_data;
use cloudevents::message::{BinaryDeserializer, Result, StructuredDeserializer};

use cloudevents::{AttributesReader, EventBuilder, EventBuilderV03, EventBuilderV10};
use std::convert::TryInto;
use test_data::*;

#[test]
fn message_v03_roundtrip_structured() -> Result<()> {
    assert_eq!(
        v03::full_json_data(),
        StructuredDeserializer::into_event(v03::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v03_roundtrip_binary() -> Result<()> {
    //TODO this code smells because we're missing a proper way in the public APIs
    // to destructure an event and rebuild it
    let wanna_be_expected = v03::full_json_data();
    let data: serde_json::Value = wanna_be_expected.data().unwrap().clone().try_into()?;
    let bytes = serde_json::to_vec(&data)?;
    let expected = EventBuilderV03::from(wanna_be_expected.clone())
        .data(wanna_be_expected.datacontenttype().unwrap(), bytes)
        .build()
        .unwrap();

    assert_eq!(
        expected,
        BinaryDeserializer::into_event(v03::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v10_roundtrip_structured() -> Result<()> {
    assert_eq!(
        v10::full_json_data(),
        StructuredDeserializer::into_event(v10::full_json_data())?
    );
    Ok(())
}

#[test]
fn message_v10_roundtrip_binary() -> Result<()> {
    //TODO this code smells because we're missing a proper way in the public APIs
    // to destructure an event and rebuild it
    let wanna_be_expected = v10::full_json_data();
    let data: serde_json::Value = wanna_be_expected
        .data()
        .cloned()
        .unwrap()
        .try_into()
        .unwrap();
    let bytes = serde_json::to_vec(&data)?;
    let expected = EventBuilderV10::from(wanna_be_expected.clone())
        .data(wanna_be_expected.datacontenttype().unwrap(), bytes)
        .build()
        .unwrap();

    assert_eq!(
        expected,
        BinaryDeserializer::into_event(v10::full_json_data())?
    );
    Ok(())
}
