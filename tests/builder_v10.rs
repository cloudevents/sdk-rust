#[macro_use]
mod util;

use chrono::{DateTime, Utc};
use cloudevents::event::{
    AttributesReader, EventBuilder, EventBuilderError, ExtensionValue, SpecVersion,
};
use cloudevents::EventBuilderV10;
use url::Url;

#[test]
fn build_event() {
    let id = "aaa";
    let source = Url::parse("http://localhost:8080").unwrap();
    let ty = "bbb";
    let subject = "francesco";
    let time: DateTime<Utc> = Utc::now();
    let extension_name = "ext";
    let extension_value = 10i64;
    let content_type = "application/json";
    let schema = Url::parse("http://localhost:8080/schema").unwrap();
    let data = serde_json::json!({
        "hello": "world"
    });

    let event = EventBuilderV10::new()
        .id(id)
        .source(source.clone())
        .ty(ty)
        .subject(subject)
        .time(time)
        .extension(extension_name, extension_value)
        .data_with_schema(content_type, schema.clone(), data.clone())
        .build()
        .unwrap();

    assert_eq!(SpecVersion::V10, event.get_specversion());
    assert_eq!(id, event.get_id());
    assert_eq!(source, event.get_source().clone());
    assert_eq!(ty, event.get_type());
    assert_eq!(subject, event.get_subject().unwrap());
    assert_eq!(time, event.get_time().unwrap().clone());
    assert_eq!(
        ExtensionValue::from(extension_value),
        event.get_extension(extension_name).unwrap().clone()
    );
    assert_eq!(content_type, event.get_datacontenttype().unwrap());
    assert_eq!(schema, event.get_dataschema().unwrap().clone());

    let event_data: serde_json::Value = event.try_get_data().unwrap().unwrap();
    assert_eq!(data, event_data);
}

#[test]
fn build_missing_id() {
    let res = EventBuilderV10::new()
        .source("http://localhost:8080")
        .build();
    assert_match_pattern!(
        res,
        Err(EventBuilderError::MissingRequiredAttribute {
            attribute_name: "id"
        })
    );
}

#[test]
fn source_invalid_url() {
    let res = EventBuilderV10::new().source("").build();
    assert_match_pattern!(
        res,
        Err(EventBuilderError::ParseUrlError {
            attribute_name: "source",
            ..
        })
    );
}

#[test]
fn default_builds() {
    let res = EventBuilderV10::default().build();
    assert_match_pattern!(res, Ok(_));
}
