use super::Attributes;
use crate::event::data::is_json_content_type;
use crate::event::{Data, ExtensionValue};
use chrono::{DateTime, Utc};
use serde::de::IntoDeserializer;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serializer};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};
use url::Url;

pub(crate) struct EventFormatDeserializer {}

impl crate::event::format::EventFormatDeserializer for EventFormatDeserializer {
    fn deserialize_attributes<E: serde::de::Error>(
        map: &mut BTreeMap<String, Value>,
    ) -> Result<crate::event::Attributes, E> {
        Ok(crate::event::Attributes::V10(Attributes {
            id: parse_field!(map, "id", String, E)?,
            ty: parse_field!(map, "type", String, E)?,
            source: parse_field!(map, "source", String, E, Url::parse)?,
            datacontenttype: parse_optional_field!(map, "datacontenttype", String, E)?,
            dataschema: parse_optional_field!(map, "dataschema", String, E, Url::parse)?,
            subject: parse_optional_field!(map, "subject", String, E)?,
            time: parse_optional_field!(map, "time", String, E, |s| DateTime::parse_from_rfc3339(
                s
            )
            .map(DateTime::<Utc>::from))?,
        }))
    }

    fn deserialize_data<E: serde::de::Error>(
        content_type: &str,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Option<Data>, E> {
        let data = map.remove("data");
        let data_base64 = map.remove("data_base64");

        let is_json = is_json_content_type(content_type);

        Ok(match (data, data_base64, is_json) {
            (Some(d), None, true) => Some(Data::Json(parse_data_json!(d, E)?)),
            (Some(d), None, false) => Some(Data::String(parse_data_string!(d, E)?)),
            (None, Some(d), true) => Some(Data::Json(parse_json_data_base64!(d, E)?)),
            (None, Some(d), false) => Some(Data::Binary(parse_data_base64!(d, E)?)),
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
        let num =
            3 + if attributes.datacontenttype.is_some() {
                1
            } else {
                0
            } + if attributes.dataschema.is_some() {
                1
            } else {
                0
            } + if attributes.subject.is_some() { 1 } else { 0 }
                + if attributes.time.is_some() { 1 } else { 0 }
                + if data.is_some() { 1 } else { 0 }
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
