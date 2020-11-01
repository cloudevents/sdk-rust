use std::fmt::Debug;

/// Represents one of the possible [message encodings/modes](https://github.com/cloudevents/spec/blob/v1.0/spec.md#message).
#[derive(PartialEq, Eq, Debug)]
pub enum Encoding {
    /// Represents the _structured-mode message_.
    STRUCTURED,
    /// Represents the _binary-mode message_.
    BINARY,
    /// Represents a non-CloudEvent or a malformed CloudEvent that cannot be recognized by the SDK.
    UNKNOWN,
}
