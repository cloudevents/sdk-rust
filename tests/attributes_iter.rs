mod test_data;
use cloudevents::event::AttributeValue;
use cloudevents::event::SpecVersion;
use test_data::*;

#[test]
fn iter_v10_test() {
    let in_event = v10::full_no_data();
    let mut iter_v10 = in_event.iter_attributes();

    assert_eq!(
        ("specversion", AttributeValue::SpecVersion(SpecVersion::V10)),
        iter_v10.next().unwrap()
    );
}

#[test]
fn iter_v03_test() {
    let in_event = v03::full_json_data();
    let mut iter_v03 = in_event.iter_attributes();

    assert_eq!(
        ("specversion", AttributeValue::SpecVersion(SpecVersion::V03)),
        iter_v03.next().unwrap()
    );
}
