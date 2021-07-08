use super::Event;
use snafu::Snafu;

/// Trait to implement a builder for [`Event`]:
/// ```
/// use cloudevents::event::{EventBuilderV10, EventBuilder};
/// use chrono::Utc;
/// use url::Url;
///
/// let event = EventBuilderV10::new()
///     .id("my_event.my_application")
///     .source("http://localhost:8080")
///     .ty("example.demo")
///     .time(Utc::now())
///     .build()
///     .unwrap();
/// ```
///
/// You can create an [`EventBuilder`] starting from an existing [`Event`] using the [`From`] trait.
/// You can create a default [`EventBuilder`] setting default values for some attributes.
pub trait EventBuilder
where
    Self: Clone + Sized + From<Event> + Default,
{
    /// Create a new empty builder
    fn new() -> Self;

    /// Build [`Event`]
    fn build(self) -> Result<Event, Error>;
}

/// Represents an error during build process
#[derive(Debug, Snafu, Clone)]
pub enum Error {
    #[snafu(display("Missing required attribute {}", attribute_name))]
    MissingRequiredAttribute { attribute_name: &'static str },
    #[snafu(display(
        "Error while setting attribute '{}' with timestamp type: {}",
        attribute_name,
        source
    ))]
    ParseTimeError {
        attribute_name: &'static str,
        source: chrono::ParseError,
    },
    #[snafu(display(
        "Error while setting attribute '{}' with uri type: {}",
        attribute_name,
        source
    ))]
    ParseUrlError {
        attribute_name: &'static str,
        source: url::ParseError,
    },
    #[snafu(display(
        "Invalid value setting attribute '{}' with uriref type",
        attribute_name,
    ))]
    InvalidUriRefError { attribute_name: &'static str },
}

#[cfg(test)]
mod tests {
    use crate::test::fixtures;
    use crate::Event;
    use crate::EventBuilder;
    use crate::EventBuilderV03;
    use crate::EventBuilderV10;
    use claim::*;
    use rstest::rstest;
    use serde_json::{json, Value};
    use serde_yaml;

    /// Test conversions

    #[test]
    fn v10_to_v03() {
        let in_event = fixtures::v10::full_json_data();
        let out_event = EventBuilderV03::from(in_event).build().unwrap();
        assert_eq!(fixtures::v03::full_json_data(), out_event)
    }

    #[test]
    fn v03_to_v10() {
        let in_event = fixtures::v03::full_json_data();
        let out_event = EventBuilderV10::from(in_event).build().unwrap();
        assert_eq!(fixtures::v10::full_json_data(), out_event)
    }

    /// Test YAML
    /// This test checks if the usage of serde_json::Value makes the Deserialize implementation incompatible with
    /// other Deserializers
    #[test]
    fn deserialize_yaml_should_succeed() {
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

    /// Test Json
    /// This test is a parametrized test that uses data from tests/test_data
    #[rstest(
        in_event,
        out_json,
        case::minimal_v03(fixtures::v03::minimal(), fixtures::v03::minimal_json()),
        case::full_v03_no_data(fixtures::v03::full_no_data(), fixtures::v03::full_no_data_json()),
        case::full_v03_with_json_data(
            fixtures::v03::full_json_data(),
            fixtures::v03::full_json_data_json()
        ),
        case::full_v03_with_xml_string_data(
            fixtures::v03::full_xml_string_data(),
            fixtures::v03::full_xml_string_data_json()
        ),
        case::full_v03_with_xml_base64_data(
            fixtures::v03::full_xml_binary_data(),
            fixtures::v03::full_xml_base64_data_json()
        ),
        case::minimal_v10(fixtures::v10::minimal(), fixtures::v10::minimal_json()),
        case::full_v10_no_data(fixtures::v10::full_no_data(), fixtures::v10::full_no_data_json()),
        case::full_v10_with_json_data(
            fixtures::v10::full_json_data(),
            fixtures::v10::full_json_data_json()
        ),
        case::full_v10_with_xml_string_data(
            fixtures::v10::full_xml_string_data(),
            fixtures::v10::full_xml_string_data_json()
        ),
        case::full_v10_with_xml_base64_data(
            fixtures::v10::full_xml_binary_data(),
            fixtures::v10::full_xml_base64_data_json()
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
        case::minimal_v03(fixtures::v03::minimal_json(), fixtures::v03::minimal()),
        case::full_v03_no_data(fixtures::v03::full_no_data_json(), fixtures::v03::full_no_data()),
        case::full_v03_with_json_data(
            fixtures::v03::full_json_data_json(),
            fixtures::v03::full_json_data()
        ),
        case::full_v03_with_json_base64_data(
            fixtures::v03::full_json_base64_data_json(),
            fixtures::v03::full_json_data()
        ),
        case::full_v03_with_xml_string_data(
            fixtures::v03::full_xml_string_data_json(),
            fixtures::v03::full_xml_string_data()
        ),
        case::full_v03_with_xml_base64_data(
            fixtures::v03::full_xml_base64_data_json(),
            fixtures::v03::full_xml_binary_data()
        ),
        case::minimal_v10(fixtures::v10::minimal_json(), fixtures::v10::minimal()),
        case::full_v10_no_data(fixtures::v10::full_no_data_json(), fixtures::v10::full_no_data()),
        case::full_v10_with_json_data(
            fixtures::v10::full_json_data_json(),
            fixtures::v10::full_json_data()
        ),
        case::full_v10_with_json_base64_data(
            fixtures::v10::full_json_base64_data_json(),
            fixtures::v10::full_json_data()
        ),
        case::full_v10_with_xml_string_data(
            fixtures::v10::full_xml_string_data_json(),
            fixtures::v10::full_xml_string_data()
        ),
        case::full_v10_with_xml_base64_data(
            fixtures::v10::full_xml_base64_data_json(),
            fixtures::v10::full_xml_binary_data()
        )
    )]
    fn deserialize_json_should_succeed(in_json: Value, out_event: Event) {
        let deserialize_result: Result<Event, serde_json::Error> = serde_json::from_value(in_json);
        assert_ok!(&deserialize_result);
        let deserialize_json = deserialize_result.unwrap();
        assert_eq!(deserialize_json, out_event)
    }

    #[test]
    fn deserialize_with_null_attribute() {
        let in_json = json!({
            "specversion" : "1.0",
            "type" : "com.example.someevent",
            "source" : "/mycontext",
            "id" : "A234-1234-1234",
            "time" : null,
            "comexampleextension1" : "value",
            "comexampleothervalue" : 5,
            "datacontenttype" : "text/xml",
            "data" : "<much wow=\"xml\"/>"
        });

        let out_event = EventBuilderV10::new()
            .ty("com.example.someevent")
            .source("/mycontext")
            .id("A234-1234-1234")
            .data("text/xml", "<much wow=\"xml\"/>")
            .extension("comexampleextension1", "value")
            .extension("comexampleothervalue", 5)
            .build()
            .unwrap();

        let deserialize_result: Result<Event, serde_json::Error> = serde_json::from_value(in_json);
        assert_ok!(&deserialize_result);
        let deserialize_json = deserialize_result.unwrap();
        assert_eq!(deserialize_json, out_event)
    }

    #[test]
    fn deserialize_with_null_ext() {
        let in_json = json!({
            "specversion" : "1.0",
            "type" : "com.example.someevent",
            "source" : "/mycontext",
            "id" : "A234-1234-1234",
            "time" : "2018-04-05T17:31:00Z",
            "comexampleextension1" : "value",
            "comexampleothervalue" : 5,
            "unsetextension": null,
            "datacontenttype" : "text/xml",
            "data" : "<much wow=\"xml\"/>"
        });

        let out_event = EventBuilderV10::new()
            .ty("com.example.someevent")
            .source("/mycontext")
            .id("A234-1234-1234")
            .time("2018-04-05T17:31:00Z")
            .data("text/xml", "<much wow=\"xml\"/>")
            .extension("comexampleextension1", "value")
            .extension("comexampleothervalue", 5)
            .build()
            .unwrap();

        let deserialize_result: Result<Event, serde_json::Error> = serde_json::from_value(in_json);
        assert_ok!(&deserialize_result);
        let deserialize_json = deserialize_result.unwrap();
        assert_eq!(deserialize_json, out_event)
    }
}
