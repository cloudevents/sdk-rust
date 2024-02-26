use crate::{
    message::{Error, Result},
    Event,
};

/// Helper struct containing text data bytes of JSON serialized [Event]
///
/// Implements [`AsRef`] so it can be directly passed to [`nats::Connection`](https://docs.rs/nats/0.21.0/nats/struct.Connection.html) methods as payload.
pub struct NatsCloudEvent {
    pub payload: Vec<u8>,
}

impl AsRef<[u8]> for NatsCloudEvent {
    fn as_ref(&self) -> &[u8] {
        self.payload.as_ref()
    }
}

impl NatsCloudEvent {
    pub fn from_event(event: Event) -> Result<Self> {
        match serde_json::to_vec(&event) {
            Ok(payload) => Ok(Self { payload }),
            Err(e) => Err(Error::SerdeJsonError { source: e }),
        }
    }
}
