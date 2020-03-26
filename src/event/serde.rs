use super::{Attributes, Data, Event, EventDeseriazerV10};
use serde::de::{Error, Unexpected, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};
use crate::event::ExtensionValue;

const SPEC_VERSIONS: [&'static str; 1] = ["1.0"];

macro_rules! parse_optional_field {
    ($map:ident, $name:literal, $value_variant:ident, $error:ty) => {
        $map.remove($name)
            .map(|val| match val {
                Value::$value_variant(v) => Ok(v),
                other => Err(<$error>::invalid_type(
                    crate::event::serde::value_to_unexpected(&other),
                    &stringify!($value_variant),
                )),
            })
            .transpose()
    };
}

macro_rules! parse_field {
    ($map:ident, $name:literal, $value_variant:ident, $error:ty) => {
        parse_optional_field!($map, $name, $value_variant, $error)?
            .ok_or_else(|| <$error>::missing_field($name))
    };
}

pub(crate) trait EventDeserializer {
    fn deserialize_attributes<E: serde::de::Error>(
        &self,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Attributes, E>;

    fn deserialize_data<E: serde::de::Error>(
        &self,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Option<Data>, E>;
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let map = match Value::deserialize(deserializer)? {
            Value::Map(m) => Ok(m),
            v => Err(Error::invalid_type(value_to_unexpected(&v), &"a map")),
        }?;

        let mut map: BTreeMap<String, Value> = map
            .into_iter()
            .map(|(k, v)| match k {
                Value::String(s) => Ok((s, v)),
                k => Err(Error::invalid_type(value_to_unexpected(&k), &"a string")),
            })
            .collect::<Result<BTreeMap<String, Value>, <D as Deserializer<'de>>::Error>>()?;

        let event_deserializer =
            match parse_field!(map, "specversion", String, <D as Deserializer<'de>>::Error)?
                .as_str()
            {
                "1.0" => Ok(EventDeseriazerV10 {}),
                s => Err(<D as Deserializer<'de>>::Error::unknown_variant(
                    s,
                    &SPEC_VERSIONS,
                )),
            }?;

        let attributes = event_deserializer.deserialize_attributes(&mut map)?;
        let data = event_deserializer.deserialize_data(&mut map)?;
        let extensions = map.into_iter()
            .map(|(k, v)| Ok((k, ExtensionValue::deserialize(v.into_deserializer())?)))
            .collect::<Result<HashMap<String, ExtensionValue>, serde_value::DeserializerError>>()
            .map_err(|e| <D as Deserializer<'de>>::Error::custom(e))?;

        Ok(Event {
            attributes,
            data,
            extensions,
        })
    }
}

// This should be provided by the Value package itself
pub(crate) fn value_to_unexpected(v: &Value) -> Unexpected {
    match v {
        Value::Bool(b) => serde::de::Unexpected::Bool(*b),
        Value::U8(n) => serde::de::Unexpected::Unsigned(*n as u64),
        Value::U16(n) => serde::de::Unexpected::Unsigned(*n as u64),
        Value::U32(n) => serde::de::Unexpected::Unsigned(*n as u64),
        Value::U64(n) => serde::de::Unexpected::Unsigned(*n),
        Value::I8(n) => serde::de::Unexpected::Signed(*n as i64),
        Value::I16(n) => serde::de::Unexpected::Signed(*n as i64),
        Value::I32(n) => serde::de::Unexpected::Signed(*n as i64),
        Value::I64(n) => serde::de::Unexpected::Signed(*n),
        Value::F32(n) => serde::de::Unexpected::Float(*n as f64),
        Value::F64(n) => serde::de::Unexpected::Float(*n),
        Value::Char(c) => serde::de::Unexpected::Char(*c),
        Value::String(s) => serde::de::Unexpected::Str(s),
        Value::Unit => serde::de::Unexpected::Unit,
        Value::Option(_) => serde::de::Unexpected::Option,
        Value::Newtype(_) => serde::de::Unexpected::NewtypeStruct,
        Value::Seq(_) => serde::de::Unexpected::Seq,
        Value::Map(_) => serde::de::Unexpected::Map,
        Value::Bytes(b) => serde::de::Unexpected::Bytes(b),
    }
}
