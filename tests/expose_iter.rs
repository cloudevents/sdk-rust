mod test_data;
use test_data::*;
use cloudevents::event::{AttributesReader, AttributeValue, IterAttribute};

#[test]
fn iter_v10_test() {
    let in_event = v10::full_json_data();
    let iter = in_event.attributes_iter();
    
    assert_eq!(iter.get_id(),"0001");

    if let IterAttribute::IterV10(mut b) = iter {
            for i in b {
            println!("{:?}",i);
        }
        assert_eq!(("id", AttributeValue::String("0001")), b.next().unwrap());
    };
}
