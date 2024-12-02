use http::header::{AsHeaderName, HeaderMap, HeaderName, HeaderValue};
use http_0_2 as http;

/// Any http library should be able to use the
/// [`to_event`](super::to_event) function with an implementation of
/// this trait.
pub trait Headers<'a> {
    type Iterator: Iterator<Item = (&'a HeaderName, &'a HeaderValue)>;
    fn get<K: AsHeaderName>(&self, name: K) -> Option<&HeaderValue>;
    fn iter(&'a self) -> Self::Iterator;
}

/// Implemention for the HeaderMap used by warp/reqwest
impl<'a> Headers<'a> for HeaderMap<HeaderValue> {
    type Iterator = http::header::Iter<'a, HeaderValue>;
    fn get<K: AsHeaderName>(&self, name: K) -> Option<&HeaderValue> {
        self.get(name)
    }
    fn iter(&'a self) -> Self::Iterator {
        self.iter()
    }
}
