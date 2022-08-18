
// Required
pub(super) const ID: &str = "id";
pub(super) const SOURCE: &str = "source";
pub(super) const SPECVERSION: &str = "specversion";
pub(super) const TYPE: &str = "type";

// Optional
pub(super) const DATACONTENTTYPE: &str = "datacontenttype";
pub(super) const DATASCHEMA: &str = "dataschema";
pub(super) const SUBJECT: &str = "subject";
pub(super) const TIME: &str = "time";

pub(super) mod prefixed {
    // Required
    pub const ID: &str = "cloudEvents:id";
    pub const SOURCE: &str = "cloudEvents:source";
    pub const SPECVERSION: &str = "cloudEvents:specversion";
    pub const TYPE: &str = "cloudEvents:type";

    // Optional
    pub const DATASCHEMA: &str = "cloudEvents:dataschema";
    pub const SUBJECT: &str = "cloudEvents:subject";
    pub const TIME: &str = "cloudEvents:time";
}