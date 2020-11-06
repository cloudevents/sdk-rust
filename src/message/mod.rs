//! Provides facilities to implement Protocol Bindings.
//!
//! Note: these APIs should be considered unstable and subject to changes.

mod deserializer;
mod encoding;
mod error;
mod no_std_io;
mod serializer;
mod types;

pub use deserializer::*;
pub use encoding::*;
pub use error::*;
pub use no_std_io::{IoError, Read, Write};
pub use serializer::*;
pub use types::MessageAttributeValue;
