extern crate serde;
extern crate serde_json;
extern crate serde_value;
extern crate snafu;

/// Provides [`Event`] data structure, [`EventBuilder`] and other facilities to work with [`Event`]
pub mod event;
/// Provides facilities to implement Protocol Bindings
pub mod message;

pub use event::Event;
pub use event::{AttributesReader, AttributesWriter};
pub use event::EventBuilder;
