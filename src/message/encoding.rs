use std::fmt::Debug;

#[derive(PartialEq, Debug)]
pub enum Encoding {
    STRUCTURED,
    BINARY,
    UNKNOWN
}
