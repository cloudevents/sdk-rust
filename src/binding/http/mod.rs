pub static SPEC_VERSION_HEADER: &str = "ce-specversion";

pub fn header_prefix(name: &str) -> String {
    super::header_prefix("ce-", name)
}
