mod attributes;
mod builder;
mod serde;

pub(crate) use crate::event::v10::serde::EventDeserializer;
pub use attributes::Attributes;
pub use builder::EventBuilder;
