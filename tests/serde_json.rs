use claim::*;
use cloudevents::Event;
use rstest::rstest;
use serde_json::Value;

mod test_data;
use test_data::*;

#[rstest(
    event,
    json,
    case::minimal_v1(minimal_v1(), minimal_v1_json()),
    case::full_v1_no_data(full_v1_no_data(), full_v1_no_data_json()),
    case::full_v1_with_json_data(full_v1_json_data(), full_v1_json_data_json()),
    case::full_v1_with_base64_data(full_v1_binary_data(), full_v1_base64_data_json())
)]
fn serialize_deserialize_should_succeed(event: Event, json: Value) {
    let serialize_result = serde_json::to_value(event.clone());
    assert_ok!(&serialize_result);
    let actual_json = serialize_result.unwrap();
    assert_eq!(&actual_json, &json);
    let deserialize_result: Result<Event, serde_json::Error> = serde_json::from_value(actual_json);
    assert_ok!(&deserialize_result);
    let deserialize_json = deserialize_result.unwrap();
    assert_eq!(deserialize_json, event)
}


#[cfg(test)]
mod tests {
    use cloudevents::EventBuilder;
    #[test]
    fn serialize_full_event_to_json_data() {
        let event = EventBuilder::v10()
            .id("id")
            .ty("test type")
            .source("http://localhost:8080")
            .build();
        let deserialized_event = serde_json::to_value(event);
        let json = "{\"type\":\"test type\",\"specversion\":\"1.0\",\"source\":\"http://localhost:8080\",\"id\":\"id\",\"datacontenttype\":\"application/json\",\"data\":\"\\\"test\\\"\"}";
        assert_eq!(json, deserialized_event.unwrap());
    }
}
