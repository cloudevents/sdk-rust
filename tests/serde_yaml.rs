use claim::*;
use cloudevents::{Event, EventBuilder, EventBuilderV10};
use serde_yaml;

mod test_data;
use test_data::*;

/// This test checks if the usage of serde_json::Value makes the Deserialize implementation incompatible with
/// other Deserializers
#[test]
fn deserialize_should_succeed() {
    let input = r#"
    id: aaa
    type: bbb
    source: http://localhost
    datacontenttype: application/json
    data: true
    specversion: "1.0"
    "#;

    let expected = EventBuilderV10::new()
        .id("aaa")
        .ty("bbb")
        .source("http://localhost")
        .data("application/json", serde_json::Value::Bool(true))
        .build()
        .unwrap();

    let deserialize_result: Result<Event, serde_yaml::Error> = serde_yaml::from_str(input);
    assert_ok!(&deserialize_result);
    let deserialized = deserialize_result.unwrap();
    assert_eq!(deserialized, expected)
}
