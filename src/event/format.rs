use super::{
    Attributes, Data, Event, EventFormatDeserializerV03, EventFormatDeserializerV10,
    EventFormatSerializerV03, EventFormatSerializerV10,
};
use crate::event::{AttributesReader, ExtensionValue};
use base64::prelude::*;
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::HashMap;

macro_rules! parse_field {
    ($value:expr, $target_type:ty, $error:ty) => {
        <$target_type>::deserialize($value.into_deserializer()).map_err(<$error>::custom)
    };

    ($value:expr, $target_type:ty, $error:ty, $mapper:expr) => {
        <$target_type>::deserialize($value.into_deserializer())
            .map_err(<$error>::custom)
            .and_then(|v| $mapper(v).map_err(<$error>::custom))
    };
}

macro_rules! extract_optional_field {
    ($map:ident, $name:literal, $target_type:ty, $error:ty) => {
        $map.remove($name)
            .filter(|v| !v.is_null())
            .map(|v| parse_field!(v, $target_type, $error))
            .transpose()
    };

    ($map:ident, $name:literal, $target_type:ty, $error:ty, $mapper:expr) => {
        $map.remove($name)
            .filter(|v| !v.is_null())
            .map(|v| parse_field!(v, $target_type, $error, $mapper))
            .transpose()
    };
}

macro_rules! extract_field {
    ($map:ident, $name:literal, $target_type:ty, $error:ty) => {
        extract_optional_field!($map, $name, $target_type, $error)?
            .ok_or_else(|| <$error>::missing_field($name))
    };

    ($map:ident, $name:literal, $target_type:ty, $error:ty, $mapper:expr) => {
        extract_optional_field!($map, $name, $target_type, $error, $mapper)?
            .ok_or_else(|| <$error>::missing_field($name))
    };
}

pub fn parse_data_json<E: serde::de::Error>(v: Value) -> Result<Value, E> {
    Value::deserialize(v.into_deserializer()).map_err(E::custom)
}

pub fn parse_data_string<E: serde::de::Error>(v: Value) -> Result<String, E> {
    parse_field!(v, String, E)
}

pub fn parse_data_base64<E: serde::de::Error>(v: Value) -> Result<Vec<u8>, E> {
    parse_field!(v, String, E).and_then(|s| {
        BASE64_STANDARD
            .decode(s)
            .map_err(|e| E::custom(format_args!("decode error `{}`", e)))
    })
}

pub fn parse_data_base64_json<E: serde::de::Error>(v: Value) -> Result<Value, E> {
    let data = parse_data_base64(v)?;
    serde_json::from_slice(&data).map_err(E::custom)
}

pub(crate) trait EventFormatDeserializer {
    fn deserialize_attributes<E: serde::de::Error>(
        map: &mut Map<String, Value>,
    ) -> Result<Attributes, E>;

    fn deserialize_data<E: serde::de::Error>(
        content_type: &str,
        map: &mut Map<String, Value>,
    ) -> Result<Option<Data>, E>;

    fn deserialize_event<E: serde::de::Error>(mut map: Map<String, Value>) -> Result<Event, E> {
        let attributes = Self::deserialize_attributes(&mut map)?;
        let data = Self::deserialize_data(
            attributes.datacontenttype().unwrap_or("application/json"),
            &mut map,
        )?;
        let extensions = map
            .into_iter()
            .filter(|v| !v.1.is_null())
            .map(|(k, v)| {
                Ok((
                    k,
                    ExtensionValue::deserialize(v.into_deserializer()).map_err(E::custom)?,
                ))
            })
            .collect::<Result<HashMap<String, ExtensionValue>, E>>()?;

        Ok(Event {
            attributes,
            data,
            extensions,
        })
    }
}

pub(crate) trait EventFormatSerializer<S: Serializer, A: Sized> {
    fn serialize(
        attributes: &A,
        data: &Option<Data>,
        extensions: &HashMap<String, ExtensionValue>,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>;
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let root_value = Value::deserialize(deserializer)?;
        let mut map: Map<String, Value> =
            Map::deserialize(root_value.into_deserializer()).map_err(D::Error::custom)?;

        match extract_field!(map, "specversion", String, <D as Deserializer<'de>>::Error)?.as_str()
        {
            "0.3" => EventFormatDeserializerV03::deserialize_event(map),
            "1.0" => EventFormatDeserializerV10::deserialize_event(map),
            s => Err(D::Error::unknown_variant(
                s,
                &super::spec_version::SPEC_VERSIONS,
            )),
        }
    }
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match &self.attributes {
            Attributes::V03(a) => {
                EventFormatSerializerV03::serialize(a, &self.data, &self.extensions, serializer)
            }
            Attributes::V10(a) => {
                EventFormatSerializerV10::serialize(a, &self.data, &self.extensions, serializer)
            }
        }
    }
}
