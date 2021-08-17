use std::{cell::RefCell, rc::Rc};

use crate::binding::{
    http::{header_prefix, SPEC_VERSION_HEADER},
    CLOUDEVENTS_JSON_HEADER,
};
use crate::event::SpecVersion;
use crate::message::{BinarySerializer, MessageAttributeValue, Result, StructuredSerializer};

macro_rules! str_to_header_value {
    ($header_value:expr) => {
        http::header::HeaderValue::from_str(&$header_value.to_string()).map_err(|e| {
            crate::message::Error::Other {
                source: Box::new(e),
            }
        })
    };
}

pub trait Builder<R> {
    fn header(&mut self, key: &str, value: http::header::HeaderValue);
    fn body(&mut self, bytes: Vec<u8>) -> Result<R>;
    fn finish(&mut self) -> Result<R>;
}

pub struct Serializer<T> {
    builder: Rc<RefCell<dyn Builder<T>>>,
}

impl<T> Serializer<T> {
    pub fn new<B: Builder<T> + 'static>(delegate: B) -> Serializer<T> {
        let builder = Rc::new(RefCell::new(delegate));
        Serializer { builder }
    }
}

impl<T> BinarySerializer<T> for Serializer<T> {
    fn set_spec_version(self, spec_version: SpecVersion) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(SPEC_VERSION_HEADER, str_to_header_value!(spec_version)?);
        Ok(self)
    }

    fn set_attribute(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn set_extension(self, name: &str, value: MessageAttributeValue) -> Result<Self> {
        self.builder
            .borrow_mut()
            .header(&header_prefix(name), str_to_header_value!(value)?);
        Ok(self)
    }

    fn end_with_data(self, bytes: Vec<u8>) -> Result<T> {
        self.builder.borrow_mut().body(bytes)
    }

    fn end(self) -> Result<T> {
        self.builder.borrow_mut().finish()
    }
}

impl<T> StructuredSerializer<T> for Serializer<T> {
    fn set_structured_event(self, bytes: Vec<u8>) -> Result<T> {
        let mut builder = self.builder.borrow_mut();
        builder.header(
            http::header::CONTENT_TYPE.as_str(),
            http::HeaderValue::from_static(CLOUDEVENTS_JSON_HEADER),
        );
        builder.body(bytes)
    }
}
