use std::convert::TryFrom;
use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub enum SpecVersion {
    V03,
    V10,
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecVersion::V03 => write!(f, "0.3"),
            SpecVersion::V10 => write!(f, "1.0"),
        }
    }
}

impl TryFrom<String> for SpecVersion {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "0.3" => Ok(SpecVersion::V03),
            "1.0" => Ok(SpecVersion::V10),
            _ => Err(format!("Invalid specversion {}", value)),
        }
    }
}
