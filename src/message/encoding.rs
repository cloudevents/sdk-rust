use std::fmt::Debug;

/// Represents one of the possible [message encodings/modes](https://github.com/cloudevents/spec/blob/v1.0/spec.md#message)
#[derive(PartialEq, Debug)]
pub enum Encoding {
    STRUCTURED,
    BINARY,
    UNKNOWN,
}
