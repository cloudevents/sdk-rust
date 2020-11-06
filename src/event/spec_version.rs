use super::{v03, v10};
use serde::export::Formatter; // double check
use snafu::Snafu;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;
use std::prelude::v1::*;

pub(crate) const SPEC_VERSIONS: [&str; 2] = ["0.3", "1.0"];

/// CloudEvent specification version.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum SpecVersion {
    /// CloudEvents v0.3
    V03,
    /// CloudEvents v1.0
    V10,
}

impl SpecVersion {
    /// Returns the string representation of [`SpecVersion`].
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            SpecVersion::V03 => "0.3",
            SpecVersion::V10 => "1.0",
        }
    }

    /// Get all attribute names for this [`SpecVersion`].
    #[inline]
    pub fn attribute_names(&self) -> &'static [&'static str] {
        match self {
            SpecVersion::V03 => &v03::ATTRIBUTE_NAMES,
            SpecVersion::V10 => &v10::ATTRIBUTE_NAMES,
        }
    }
    /// Get all attribute names for all specification versions.
    /// Note that the result iterator could contain duplicate entries.
    pub fn all_attribute_names() -> impl Iterator<Item = &'static str> {
        vec![SpecVersion::V03, SpecVersion::V10]
            .into_iter()
            .flat_map(|s| s.attribute_names().to_owned().into_iter())
    }
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error representing an unknown [`SpecVersion`] string identifier
#[derive(Debug)]
pub struct UnknownSpecVersion {
    spec_version_value: String,
}

impl fmt::Display for UnknownSpecVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid specversion {}", self.spec_version_value)
    }
}

impl snafu::Error for UnknownSpecVersion {}

impl TryFrom<&str> for SpecVersion {
    type Error = UnknownSpecVersion;

    fn try_from(value: &str) -> core::result::Result<Self, UnknownSpecVersion> {
        match value {
            "0.3" => Ok(SpecVersion::V03),
            "1.0" => Ok(SpecVersion::V10),
            _ => Err(UnknownSpecVersion {
                spec_version_value: value.to_string(),
            }),
        }
    }
}
