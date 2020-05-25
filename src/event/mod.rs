mod attributes;
mod builder;
mod data;
mod event;
mod extensions;
#[macro_use]
mod format;
mod message;
mod spec_version;

pub use attributes::Attributes;
pub use attributes::{AttributesReader, AttributesWriter};
pub use builder::EventBuilder;
pub use data::Data;
pub use event::Event;
pub use extensions::ExtensionValue;
pub use spec_version::InvalidSpecVersion;
pub use spec_version::SpecVersion;

mod v03;

pub use v03::Attributes as AttributesV03;
pub use v03::EventBuilder as EventBuilderV03;
pub(crate) use v03::EventFormatDeserializer as EventFormatDeserializerV03;
pub(crate) use v03::EventFormatSerializer as EventFormatSerializerV03;

mod v10;

pub use v10::Attributes as AttributesV10;
pub use v10::EventBuilder as EventBuilderV10;
pub(crate) use v10::EventFormatDeserializer as EventFormatDeserializerV10;
pub(crate) use v10::EventFormatSerializer as EventFormatSerializerV10;
