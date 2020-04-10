use super::{AttributesV03, AttributesV10, SpecVersion};
use chrono::{DateTime, Utc};

macro_rules! attributes_def {
    // struct attributes expansion
    (@struct_gen $struct_name:ident {$(,)*} -> { $($out:tt)* }) => {
        #[derive(PartialEq, Debug, Clone)]
        pub struct $struct_name {
            $($out)*
        }
    };
    (@struct_gen $struct_name:ident {$element_name:ident $(as $_element_lit:literal)?: $element_ty:ty $({$($opts:tt)*})?, $($tail:tt)*} -> {}) => {
        attributes_def!(@struct_gen $struct_name { $($tail)* } -> {pub(crate) $element_name: $element_ty});
    };
    (@struct_gen $struct_name:ident {$element_name:ident $(as $_element_lit:literal)?: $element_ty:ty $({$($opts:tt)*})?, $($tail:tt)*} -> {$($out:tt)*}) => {
        attributes_def!(@struct_gen $struct_name { $($tail)* } -> {$($out)*, pub(crate) $element_name: $element_ty});
    };

    // count attributes
    (@count_attrs ) => {0usize};
    (@count_attrs $_element_name:ident $(as $_element_lit:literal)?: $element_ty:ty $({$($opts:tt)*})?, $($tail:tt)*) => {1usize + attributes_def!{@count_attrs $($tail)* }};

    // names expansion
    (@attributes_gen {} -> {$($out:expr)*} ) => {
        [$($out),*]
    };
    (@attributes_gen { $element_name:ident: $_element_ty:ty $({$($_opts:tt)*})?, $($tail:tt)* } -> {$($out:tt)*}) => {
        attributes_def!(@attributes_gen {$($tail)*} -> {$($out)* stringify!($element_name)})
    };
    (@attributes_gen { $_element_name:ident as $element_lit:literal: $_element_ty:ty $({$($_opts:tt)*})? , $($tail:tt)* } -> {$($out:tt)*}) => {
        attributes_def!(@attributes_gen {$($tail)*} -> {$($out)* $element_lit})
    };

    // attribute names expansion
    (@attribute_names_gen $attr_vec_name:ident { $($attrs:tt)* }) => {
        pub const $attr_vec_name: [&'static str; attributes_def!(@count_attrs $($attrs)*)] = attributes_def!(@attributes_gen { $($attrs)* } -> {});
    };

    // default trait implementation expansion
    (@default_gen $struct_name:ident {$(,)*} -> { $($out:tt)* }) => {
        impl Default for $struct_name {
            fn default() -> Self {
                $struct_name {
                    $($out)*
                }
            }
        }
    };
    (@default_gen $struct_name:ident {
        $element_name:ident $(as $_element_lit:literal)?: $_element_ty:ty, $($tail:tt)*
    } -> { $($out:tt)* } ) => {
        attributes_def!(@default_gen $struct_name { $($tail)* } -> {$($out)* $element_name: attributes_def!(@default_gen_walk_opts {}), });
    };
    (@default_gen $struct_name:ident {
        $element_name:ident $(as $_element_lit:literal)?: $_element_ty:ty {$($opts:tt)*}, $($tail:tt)*
    } -> { $($out:tt)* } ) => {
        attributes_def!(@default_gen $struct_name { $($tail)* } -> {$($out)* $element_name: attributes_def!(@default_gen_walk_opts {$($opts)*}), });
    };
    (@default_gen_walk_opts {default: $default_expr:expr, $($tail:tt)*}) => {
        $default_expr
    };
    (@default_gen_walk_opts {$_opt_key:ident: $_opt_val:ident, $($tail:tt)*}) => {
        attributes_def!(@default_gen_walk_opts {$($tail)*})
    };
    (@default_gen_walk_opts {}) => {
        Default::default()
    };

    // Real macro input
    ($struct_name:ident, $attr_names:ident, { $($tt:tt)* }) => {
        attributes_def!(@attribute_names_gen $attr_names { $($tt)* });
        attributes_def!(@struct_gen $struct_name { $($tt)* } -> {});
        attributes_def!(@default_gen $struct_name { $($tt)* } -> {});
    };
}

/// Trait to get [CloudEvents Context attributes](https://github.com/cloudevents/spec/blob/master/spec.md#context-attributes).
pub trait AttributesReader {
    /// Get the [id](https://github.com/cloudevents/spec/blob/master/spec.md#id).
    fn get_id(&self) -> &str;
    /// Get the [source](https://github.com/cloudevents/spec/blob/master/spec.md#source-1).
    fn get_source(&self) -> &str;
    /// Get the [specversion](https://github.com/cloudevents/spec/blob/master/spec.md#specversion).
    fn get_specversion(&self) -> SpecVersion;
    /// Get the [type](https://github.com/cloudevents/spec/blob/master/spec.md#type).
    fn get_type(&self) -> &str;
    /// Get the [datacontenttype](https://github.com/cloudevents/spec/blob/master/spec.md#datacontenttype).
    fn get_datacontenttype(&self) -> Option<&str>;
    /// Get the [dataschema](https://github.com/cloudevents/spec/blob/master/spec.md#dataschema).
    fn get_dataschema(&self) -> Option<&str>;
    /// Get the [subject](https://github.com/cloudevents/spec/blob/master/spec.md#subject).
    fn get_subject(&self) -> Option<&str>;
    /// Get the [time](https://github.com/cloudevents/spec/blob/master/spec.md#time).
    fn get_time(&self) -> Option<&DateTime<Utc>>;
}

pub trait AttributesWriter {
    fn set_id(&mut self, id: impl Into<String>);
    fn set_source(&mut self, source: impl Into<String>);
    fn set_type(&mut self, ty: impl Into<String>);
    fn set_subject(&mut self, subject: Option<impl Into<String>>);
    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>);
}

pub(crate) trait AttributesConverter {
    fn into_v03(self) -> AttributesV03;
    fn into_v10(self) -> AttributesV10;
}

pub(crate) trait DataAttributesWriter {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>);
    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>);
}

#[derive(PartialEq, Debug, Clone)]
pub enum Attributes {
    V03(AttributesV03),
    V10(AttributesV10),
}

impl AttributesReader for Attributes {
    fn get_id(&self) -> &str {
        match self {
            Attributes::V03(a) => a.get_id(),
            Attributes::V10(a) => a.get_id(),
        }
    }

    fn get_source(&self) -> &str {
        match self {
            Attributes::V03(a) => a.get_source(),
            Attributes::V10(a) => a.get_source(),
        }
    }

    fn get_specversion(&self) -> SpecVersion {
        match self {
            Attributes::V03(a) => a.get_specversion(),
            Attributes::V10(a) => a.get_specversion(),
        }
    }

    fn get_type(&self) -> &str {
        match self {
            Attributes::V03(a) => a.get_type(),
            Attributes::V10(a) => a.get_type(),
        }
    }

    fn get_datacontenttype(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.get_datacontenttype(),
            Attributes::V10(a) => a.get_datacontenttype(),
        }
    }

    fn get_dataschema(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.get_dataschema(),
            Attributes::V10(a) => a.get_dataschema(),
        }
    }

    fn get_subject(&self) -> Option<&str> {
        match self {
            Attributes::V03(a) => a.get_subject(),
            Attributes::V10(a) => a.get_subject(),
        }
    }

    fn get_time(&self) -> Option<&DateTime<Utc>> {
        match self {
            Attributes::V03(a) => a.get_time(),
            Attributes::V10(a) => a.get_time(),
        }
    }
}

impl AttributesWriter for Attributes {
    fn set_id(&mut self, id: impl Into<String>) {
        match self {
            Attributes::V03(a) => a.set_id(id),
            Attributes::V10(a) => a.set_id(id),
        }
    }

    fn set_source(&mut self, source: impl Into<String>) {
        match self {
            Attributes::V03(a) => a.set_source(source),
            Attributes::V10(a) => a.set_source(source),
        }
    }

    fn set_type(&mut self, ty: impl Into<String>) {
        match self {
            Attributes::V03(a) => a.set_type(ty),
            Attributes::V10(a) => a.set_type(ty),
        }
    }

    fn set_subject(&mut self, subject: Option<impl Into<String>>) {
        match self {
            Attributes::V03(a) => a.set_subject(subject),
            Attributes::V10(a) => a.set_subject(subject),
        }
    }

    fn set_time(&mut self, time: Option<impl Into<DateTime<Utc>>>) {
        match self {
            Attributes::V03(a) => a.set_time(time),
            Attributes::V10(a) => a.set_time(time),
        }
    }
}

impl DataAttributesWriter for Attributes {
    fn set_datacontenttype(&mut self, datacontenttype: Option<impl Into<String>>) {
        match self {
            Attributes::V03(a) => a.set_datacontenttype(datacontenttype),
            Attributes::V10(a) => a.set_datacontenttype(datacontenttype),
        }
    }

    fn set_dataschema(&mut self, dataschema: Option<impl Into<String>>) {
        match self {
            Attributes::V03(a) => a.set_dataschema(dataschema),
            Attributes::V10(a) => a.set_dataschema(dataschema),
        }
    }
}

impl Attributes {
    pub fn into_v10(self) -> Self {
        match self {
            Attributes::V03(v03) => Attributes::V10(v03.into_v10()),
            _ => self,
        }
    }
    pub fn into_v03(self) -> Self {
        match self {
            Attributes::V10(v10) => Attributes::V03(v10.into_v03()),
            _ => self,
        }
    }
}
