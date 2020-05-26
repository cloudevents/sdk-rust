mod test_data;
use cloudevents::event::{EventBuilderV03, EventBuilderV10};
use test_data::*;
use cloudevents::EventBuilder;

#[test]
fn v10_to_v03() {
    let in_event = v10::full_json_data();
    let out_event = EventBuilderV03::from(in_event).build().unwrap();
    assert_eq!(v03::full_json_data(), out_event)
}

#[test]
fn v03_to_v10() {
    let in_event = v03::full_json_data();
    let out_event = EventBuilderV10::from(in_event).build().unwrap();
    assert_eq!(v10::full_json_data(), out_event)
}
