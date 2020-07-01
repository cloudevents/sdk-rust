use super::headers;


pub struct ResponseDeserializer {
    payload: &str,
    headers: Vec<(&str, &[u8])>],
}

impl ResponseDeserializer {
    pub fn new(payload: &str, headers: Vec<(&str, &[u8])>) -> ResponseDeserializer {
        ResponseDeserializer { payload,headers }
    }
}

impl BinaryDeserializer for ResponseDeserializer {
    fn deserialize_binary<R: Sized, V: BinarySerializer<R>>(self, mut visitor: V) -> Result<R> {
        if self.encoding() != Encoding::BINARY {
            return Err(message::Error::WrongEncoding {});
        }

        let spec_version = SpecVersion::try_from(
            unwrap_optional_header!(self.headers, headers::SPEC_VERSION_HEADER).unwrap()?,
        )?;

        visitor = visitor.set_spec_version(spec_version.clone())?;

        let attributes = spec_version.attribute_names();

        for (hn, hv) in self
            .headers
            .iter()
            .filter(|(hn, _)| headers::SPEC_VERSION_HEADER.ne(hn) && hn.as_str().starts_with("ce-"))
        {
            let name = &hn.as_str()["ce-".len()..];

            if attributes.contains(&name) {
                visitor = visitor.set_attribute(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            } else {
                visitor = visitor.set_extension(
                    name,
                    MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
                )?
            }
        }

        if let Some(hv) = self.headers.get("content-type") {
            visitor = visitor.set_attribute(
                "datacontenttype",
                MessageAttributeValue::String(String::from(header_value_to_str!(hv)?)),
            )?
        }

        if self.body.len() != 0 {
            visitor.end_with_data(self.body.to_vec())
        } else {
            visitor.end()
        }
    }
}

impl MessageDeserializer for ResponseDeserializer {
    fn encoding(&self) -> Encoding {
        match (
            unwrap_optional_header!(self.headers, reqwest::header::CONTENT_TYPE)
                .map(|r| r.ok())
                .flatten()
                .map(|e| e.starts_with("application/cloudevents+json")),
            self.headers
                .get::<&'static HeaderName>(&headers::SPEC_VERSION_HEADER),
        ) {
            (Some(true), _) => Encoding::STRUCTURED,
            (_, Some(_)) => Encoding::BINARY,
            _ => Encoding::UNKNOWN,
        }
    }
}

let payload = match m.payload_view::<str>() {
    None => "",
    Some(Ok(s)) => s,
    Some(Err(e)) => {
        warn!("Error while deserializing message payload: {:?}", e);
        ""
    }
};