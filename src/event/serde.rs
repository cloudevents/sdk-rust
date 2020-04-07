use super::{Attributes, Data, Event, EventDeserializerV03, EventDeserializerV10, EventSerializerV03, EventSerializerV10};
use crate::event::ExtensionValue;
use serde::de::{Error, IntoDeserializer, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};

const SPEC_VERSIONS: [&'static str; 2] = ["0.3", "1.0"];

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
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Attributes, E>;

    fn deserialize_data<E: serde::de::Error>(
        map: &mut BTreeMap<String, Value>,
    ) -> Result<Option<Data>, E>;

    fn deserialize_event<E: serde::de::Error>(mut map: BTreeMap<String, Value>) -> Result<Event, E>
    {
        let attributes = Self::deserialize_attributes(&mut map)?;
        let data = Self::deserialize_data(&mut map)?;
        let extensions = map
            .into_iter()
            .map(|(k, v)| Ok((k, ExtensionValue::deserialize(v.into_deserializer())?)))
            .collect::<Result<HashMap<String, ExtensionValue>, serde_value::DeserializerError>>()
            .map_err(|e| E::custom(e))?;

        Ok(Event {
            attributes,
            data,
            extensions,
        })
    }
}

pub(crate) trait EventSerializer<S: Serializer, A: Sized> {
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

        match parse_field!(map, "specversion", String, <D as Deserializer<'de>>::Error)?.as_str()
            {
                "0.3" => EventDeserializerV03::deserialize_event(map),
                "1.0" => EventDeserializerV10::deserialize_event(map),
                s => Err(D::Error::unknown_variant(
                    s,
                    &SPEC_VERSIONS,
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
                EventSerializerV03::serialize(a, &self.data, &self.extensions, serializer)
            }
            Attributes::V10(a) => {
                EventSerializerV10::serialize(a, &self.data, &self.extensions, serializer)
            }
        }
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
