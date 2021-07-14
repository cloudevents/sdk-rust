macro_rules! str_to_header_value {
    ($header_value:expr) => {
        HeaderValue::from_str($header_value.to_string().as_str()).map_err(|e| {
            crate::message::Error::Other {
                source: Box::new(e),
            }
        })
    };
}
