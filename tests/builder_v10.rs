#[macro_use]
mod util;

use chrono::{DateTime, Utc};
use cloudevents::event::{
    AttributesReader, EventBuilder, EventBuilderError, ExtensionValue, SpecVersion,
};
use cloudevents::EventBuilderV10;
use std::convert::TryInto;

use cloudevents::event::UrlExtend;
#[cfg(feature = "std")]
use url::Url;
#[cfg(not(feature = "std"))]
use String as Url;

#[cfg(feature = "std")]
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

    let mut event = EventBuilderV10::new()
        .id(id)
        .source(source.clone())
        .ty(ty)
        .subject(subject)
        .time(time)
        .extension(extension_name, extension_value)
        .data_with_schema(content_type, schema.clone(), data.clone())
        .build()
        .unwrap();

    assert_eq!(SpecVersion::V10, event.specversion());
    assert_eq!(id, event.id());
    assert_eq!(source, event.source().clone());
    assert_eq!(ty, event.ty());
    assert_eq!(subject, event.subject().unwrap());
    assert_eq!(time, event.time().unwrap().clone());
    assert_eq!(
        ExtensionValue::from(extension_value),
        event.extension(extension_name).unwrap().clone()
    );
    assert_eq!(content_type, event.datacontenttype().unwrap());
    assert_eq!(schema, event.dataschema().unwrap().clone());

    let event_data: serde_json::Value = event.take_data().2.unwrap().try_into().unwrap();
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
