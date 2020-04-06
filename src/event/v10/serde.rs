use super::Attributes;
use crate::event::{Data, ExtensionValue};
use chrono::{DateTime, Utc};
use serde::de::{IntoDeserializer, Unexpected};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serializer};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};

pub(crate) struct EventDeserializer {}

impl crate::event::serde::EventDeserializer for EventDeserializer {
    fn deserialize_attributes<E: serde::de::Error>(
        &self,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<crate::event::Attributes, E> {
        Ok(crate::event::Attributes::V10(Attributes {
            id: parse_field!(map, "id", String, E)?,
            ty: parse_field!(map, "type", String, E)?,
            source: parse_field!(map, "source", String, E)?,
            datacontenttype: parse_optional_field!(map, "datacontenttype", String, E)?,
            dataschema: parse_optional_field!(map, "dataschema", String, E)?,
            subject: parse_optional_field!(map, "subject", String, E)?,
            time: parse_optional_field!(map, "time", String, E)?
                .map(|s| match DateTime::parse_from_rfc3339(&s) {
                    Ok(d) => Ok(DateTime::<Utc>::from(d)),
                    Err(e) => Err(E::invalid_value(
                        Unexpected::Str(&s),
                        &e.to_string().as_str(),
                    )),
                })
                .transpose()?,
        }))
    }

    fn deserialize_data<E: serde::de::Error>(
        &self,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Option<Data>, E> {
        let data = map.remove("data");
        let data_base64 = map.remove("data_base64");

        match (data, data_base64) {
            (Some(d), None) => Ok(Some(Data::Json(
                serde_json::Value::deserialize(d.into_deserializer()).map_err(|e| E::custom(e))?,
            ))),
            (None, Some(d)) => match d {
                Value::String(s) => Ok(Some(Data::from_base64(s.clone()).map_err(|e| {
                    E::invalid_value(Unexpected::Str(&s), &e.to_string().as_str())
                })?)),
                other => Err(E::invalid_type(
                    crate::event::serde::value_to_unexpected(&other),
                    &"a string",
                )),
            },
            (Some(_), Some(_)) => Err(E::custom("Cannot have both data and data_base64 field")),
            (None, None) => Ok(None),
        }
    }
}

pub(crate) struct EventSerializer {}

impl<S: serde::Serializer> crate::event::serde::EventSerializer<S, Attributes> for EventSerializer {
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
