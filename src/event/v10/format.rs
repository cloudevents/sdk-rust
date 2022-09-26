use super::Attributes;
use crate::event::data::is_json_content_type;
use crate::event::format::{
    parse_data_base64, parse_data_base64_json, parse_data_json, parse_data_string,
};
use crate::event::{Data, ExtensionValue};
use chrono::{DateTime, Utc};
use serde::de::IntoDeserializer;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serializer};
use serde_json::{Map, Value};
use std::collections::HashMap;
use url::Url;

pub(crate) struct EventFormatDeserializer {}

impl crate::event::format::EventFormatDeserializer for EventFormatDeserializer {
    fn deserialize_attributes<E: serde::de::Error>(
        map: &mut Map<String, Value>,
    ) -> Result<crate::event::Attributes, E> {
        Ok(crate::event::Attributes::V10(Attributes {
            id: extract_field!(map, "id", String, E)?,
            ty: extract_field!(map, "type", String, E)?,
            source: extract_field!(map, "source", String, E)?,
            datacontenttype: extract_optional_field!(map, "datacontenttype", String, E)?,
            dataschema: extract_optional_field!(map, "dataschema", String, E, |s: String| {
                Url::parse(&s)
            })?,
            subject: extract_optional_field!(map, "subject", String, E)?,
            time: extract_optional_field!(map, "time", String, E, |s: String| {
                DateTime::parse_from_rfc3339(&s).map(DateTime::<Utc>::from)
            })?,
        }))
    }

    fn deserialize_data<E: serde::de::Error>(
        content_type: &str,
        map: &mut Map<String, Value>,
    ) -> Result<Option<Data>, E> {
        let data = map.remove("data");
        let data_base64 = map.remove("data_base64");

        let is_json = is_json_content_type(content_type);

        Ok(match (data, data_base64, is_json) {
            (Some(d), None, true) => Some(Data::Json(parse_data_json(d)?)),
            (Some(d), None, false) => Some(Data::String(parse_data_string(d)?)),
            (None, Some(d), true) => match parse_data_base64_json::<E>(d.to_owned()) {
                Ok(x) => Some(Data::Json(x)),
                Err(_) => Some(Data::Binary(parse_data_base64(d)?)),
            },
            (None, Some(d), false) => Some(Data::Binary(parse_data_base64(d)?)),
            (Some(_), Some(_), _) => {
                return Err(E::custom("Cannot have both data and data_base64 field"))
            }
            (None, None, _) => None,
        })
    }
}

pub(crate) struct EventFormatSerializer {}

impl<S: serde::Serializer> crate::event::format::EventFormatSerializer<S, Attributes>
    for EventFormatSerializer
{
    fn serialize(
        attributes: &Attributes,
        data: &Option<Data>,
        extensions: &HashMap<String, ExtensionValue>,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
        let num = 4
            + [
                attributes.datacontenttype.is_some(),
                attributes.dataschema.is_some(),
                attributes.subject.is_some(),
                attributes.time.is_some(),
                data.is_some(),
            ]
            .iter()
            .filter(|&b| *b)
            .count()
            + extensions.len();

        let mut state = serializer.serialize_map(Some(num))?;
        state.serialize_entry("specversion", "1.0")?;
        state.serialize_entry("id", &attributes.id)?;
        state.serialize_entry("type", &attributes.ty)?;
        state.serialize_entry("source", &attributes.source)?;
        if let Some(datacontenttype) = &attributes.datacontenttype {
            state.serialize_entry("datacontenttype", datacontenttype)?;
        }
        if let Some(dataschema) = &attributes.dataschema {
            state.serialize_entry("dataschema", dataschema)?;
        }
        if let Some(subject) = &attributes.subject {
            state.serialize_entry("subject", subject)?;
        }
        if let Some(time) = &attributes.time {
            state.serialize_entry("time", time)?;
        }
        match data {
            Some(Data::Json(j)) => state.serialize_entry("data", j)?,
            Some(Data::String(s)) => state.serialize_entry("data", s)?,
            Some(Data::Binary(v)) => state.serialize_entry("data_base64", &base64::encode(v))?,
            _ => (),
        };
        for (k, v) in extensions {
            state.serialize_entry(k, v)?;
        }
        state.end()
    }
}
