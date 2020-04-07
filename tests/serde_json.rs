use claim::*;
use cloudevents::Event;
use rstest::rstest;
use serde_json::Value;

mod test_data;
use test_data::*;

/// This test is a parametrized test that uses data from tests/test_data
#[rstest(
    in_event,
    out_json,
    case::minimal_v1(minimal_v1(), minimal_v1_json()),
    case::full_v1_no_data(full_v1_no_data(), full_v1_no_data_json()),
    case::full_v1_with_json_data(full_v1_json_data(), full_v1_json_data_json()),
    case::full_v1_with_xml_string_data(full_v1_xml_string_data(), full_v1_xml_string_data_json()),
    case::full_v1_with_xml_base64_data(full_v1_xml_binary_data(), full_v1_xml_base64_data_json()),
)]
fn serialize_should_succeed(in_event: Event, out_json: Value) {
    // Event -> serde_json::Value
    let serialize_result = serde_json::to_value(in_event.clone());
    assert_ok!(&serialize_result);
    let actual_json = serialize_result.unwrap();
    assert_eq!(&actual_json, &out_json);

    // serde_json::Value -> String
    let actual_json_serialized = actual_json.to_string();
    assert_eq!(actual_json_serialized, out_json.to_string());

    // String -> Event
    let deserialize_result: Result<Event, serde_json::Error> =
        serde_json::from_str(&actual_json_serialized);
    assert_ok!(&deserialize_result);
    let deserialize_json = deserialize_result.unwrap();
    assert_eq!(deserialize_json, in_event)
}

/// This test is a parametrized test that uses data from tests/test_data
#[rstest(
    in_json,
    out_event,
    case::minimal_v1(minimal_v1_json(), minimal_v1()),
    case::full_v1_no_data(full_v1_no_data_json(), full_v1_no_data()),
    case::full_v1_with_json_data(full_v1_json_data_json(), full_v1_json_data()),
    case::full_v1_with_json_base64_data(full_v1_json_base64_data_json(), full_v1_json_data()),
    case::full_v1_with_xml_string_data(full_v1_xml_string_data_json(), full_v1_xml_string_data()),
    case::full_v1_with_xml_base64_data(full_v1_xml_base64_data_json(), full_v1_xml_binary_data())
)]
fn deserialize_should_succeed(in_json: Value, out_event: Event) {
    let deserialize_result: Result<Event, serde_json::Error> =
        serde_json::from_value(in_json);
    assert_ok!(&deserialize_result);
    let deserialize_json = deserialize_result.unwrap();
    assert_eq!(deserialize_json, out_event)
}

