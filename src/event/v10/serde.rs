use super::Attributes;
use crate::event::Data;
use chrono::{DateTime, Utc};
use serde::de::{IntoDeserializer, Unexpected};
use serde::Deserialize;
use serde_value::Value;
use std::collections::BTreeMap;

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
