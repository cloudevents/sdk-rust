mod attributes;
mod builder;
mod data;
mod event;
mod extensions;
#[macro_use]
mod serde;
mod spec_version;

pub use attributes::Attributes;
pub use attributes::{AttributesReader, AttributesWriter};
pub use builder::EventBuilder;
pub use data::Data;
pub use event::Event;
pub use extension_value::ExtensionValue;
pub use spec_version::SpecVersion;

mod v10;

pub use v10::Attributes as AttributesV10;
pub use v10::EventBuilder as EventBuilderV10;
pub(crate) use v10::EventDeserializer as EventDeserializerV10;
pub(crate) use v10::EventSerializer as EventSerializerV10;
