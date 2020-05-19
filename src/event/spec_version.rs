use super::{v03, v10};
use lazy_static::lazy_static;
use serde::export::Formatter;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

lazy_static! {
    /// Lazily initialized map that contains all the context attribute names per [`SpecVersion`]
    pub static ref ATTRIBUTE_NAMES: HashMap<SpecVersion, &'static [&'static str]> = {
        let mut m = HashMap::new();
        m.insert(SpecVersion::V03, &v03::ATTRIBUTE_NAMES[..]);
        m.insert(SpecVersion::V10, &v10::ATTRIBUTE_NAMES[..]);
        m
    };
}

pub(crate) const SPEC_VERSIONS: [&'static str; 2] = ["0.3", "1.0"];

/// CloudEvent specification version
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum SpecVersion {
    V03,
    V10,
}

impl SpecVersion {
    pub fn as_str(&self) -> &str {
        match self {
            SpecVersion::V03 => "0.3",
            SpecVersion::V10 => "1.0",
        }
    }
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error representing an invalid [`SpecVersion`] string identifier
#[derive(Debug)]
pub struct InvalidSpecVersion {
    spec_version_value: String,
}

impl fmt::Display for InvalidSpecVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid specversion {}", self.spec_version_value)
    }
}

impl std::error::Error for InvalidSpecVersion {}

impl TryFrom<&str> for SpecVersion {
    type Error = InvalidSpecVersion;

    fn try_from(value: &str) -> Result<Self, InvalidSpecVersion> {
        match value {
            "0.3" => Ok(SpecVersion::V03),
            "1.0" => Ok(SpecVersion::V10),
            _ => Err(InvalidSpecVersion {
                spec_version_value: value.to_string(),
            }),
        }
    }
}
