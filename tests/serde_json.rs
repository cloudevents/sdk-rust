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
    case::minimal_v03(v03::minimal(), v03::minimal_json()),
    case::full_v03_no_data(v03::full_no_data(), v03::full_no_data_json()),
    case::full_v03_with_json_data(v03::full_json_data(), v03::full_json_data_json()),
    case::full_v03_with_xml_string_data(
        v03::full_xml_string_data(),
        v03::full_xml_string_data_json()
    ),
    case::full_v03_with_xml_base64_data(
        v03::full_xml_binary_data(),
        v03::full_xml_base64_data_json()
    ),
    case::minimal_v10(v10::minimal(), v10::minimal_json()),
    case::full_v10_no_data(v10::full_no_data(), v10::full_no_data_json()),
    case::full_v10_with_json_data(v10::full_json_data(), v10::full_json_data_json()),
    case::full_v10_with_xml_string_data(
        v10::full_xml_string_data(),
        v10::full_xml_string_data_json()
    ),
    case::full_v10_with_xml_base64_data(
        v10::full_xml_binary_data(),
        v10::full_xml_base64_data_json()
    )
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
    case::minimal_v03(v03::minimal_json(), v03::minimal()),
    case::full_v03_no_data(v03::full_no_data_json(), v03::full_no_data()),
    case::full_v03_with_json_data(v03::full_json_data_json(), v03::full_json_data()),
    case::full_v03_with_json_base64_data(v03::full_json_base64_data_json(), v03::full_json_data()),
    case::full_v03_with_xml_string_data(
        v03::full_xml_string_data_json(),
        v03::full_xml_string_data()
    ),
    case::full_v03_with_xml_base64_data(
        v03::full_xml_base64_data_json(),
        v03::full_xml_binary_data()
    ),
    case::minimal_v10(v10::minimal_json(), v10::minimal()),
    case::full_v10_no_data(v10::full_no_data_json(), v10::full_no_data()),
    case::full_v10_with_json_data(v10::full_json_data_json(), v10::full_json_data()),
    case::full_v10_with_json_base64_data(v10::full_json_base64_data_json(), v10::full_json_data()),
    case::full_v10_with_xml_string_data(
        v10::full_xml_string_data_json(),
        v10::full_xml_string_data()
    ),
    case::full_v10_with_xml_base64_data(
        v10::full_xml_base64_data_json(),
        v10::full_xml_binary_data()
    )
)]
fn deserialize_should_succeed(in_json: Value, out_event: Event) {
    let deserialize_result: Result<Event, serde_json::Error> = serde_json::from_value(in_json);
    assert_ok!(&deserialize_result);
    let deserialize_json = deserialize_result.unwrap();
    assert_eq!(deserialize_json, out_event)
}
