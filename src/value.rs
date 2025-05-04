//! A GameSON value.

use std::{
    fmt::{Display, Write},
    sync::Arc,
};

use crate::{
    TypeDefinitionInstance, type_attributes::ValidateNumberTypeError,
    type_attributes_instance::TypeAttributesInstance,
};

/// A GameSON value.
///
/// The value is guaranteed to be valid for the type instance it is associated with.
#[derive(Debug, Clone)]
pub struct Value<Id, FieldName: Ord> {
    /// The type instance.
    instance: Arc<TypeDefinitionInstance<Id, FieldName>>,

    /// The value.
    value: ValueImpl<FieldName>,
}

impl<Id, FieldName: Ord> Display for Value<Id, FieldName>
where
    Id: Display,
    FieldName: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_for(&self.instance, f)
    }
}
/// An error that can occur when parsing a GameSON value.
#[derive(Debug, thiserror::Error)]
#[error("failed to parse GameSON value `{}` ({}): {path}: {err}", .instance.name, instance.id)]
pub struct ParseError<Id: Display, FieldName: Ord + Display> {
    /// The name of the type.
    instance: Arc<TypeDefinitionInstance<Id, FieldName>>,

    /// The path of the value that caused the error.
    path: ParseErrorPath,

    /// The value parse error.
    err: ParseImplError,
}

/// GameSON value parse error path.
#[derive(Debug)]
struct ParseErrorPath(Vec<ParseErrorPathSegment>);

impl Default for ParseErrorPath {
    fn default() -> Self {
        Self(Vec::with_capacity(8))
    }
}

impl Display for ParseErrorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.0 {
            segment.fmt(f)?;
        }

        Ok(())
    }
}

impl ParseErrorPath {
    /// Push a new segment to the path.
    fn push(&mut self, segment: ParseErrorPathSegment) {
        self.0.push(segment);
    }

    /// Pop the last segment from the path.
    ///
    /// If the path is empty, this function panics.
    fn pop(&mut self) {
        self.0.pop().expect("pop from empty path");
    }
}

/// A path segment for a GameSON value parse error.
#[derive(Debug)]
enum ParseErrorPathSegment {
    /// An array index.
    ArrayIndex(usize),

    /// A dictionary key.
    DictionaryKey(String),
}

impl Display for ParseErrorPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArrayIndex(index) => write!(f, "[{index}]"),
            Self::DictionaryKey(key) => write!(f, "[{key}]"),
        }
    }
}

impl<Id: Display, FieldName: Ord + Display> Value<Id, FieldName> {
    /// Parse a GameSON value from a JSON value for a specified type instance.
    pub fn parse_for(
        instance: Arc<TypeDefinitionInstance<Id, FieldName>>,
        value: serde_json::Value,
    ) -> Result<Self, ParseError<Id, FieldName>> {
        let mut path = ParseErrorPath::default();

        match ValueImpl::parse_for(&mut path, &instance, value) {
            Ok(value) => Ok(Self { instance, value }),
            Err(err) => {
                return Err(ParseError {
                    instance,
                    path,
                    err,
                });
            }
        }
    }
}

/// A GameSON value implementation.
#[derive(Debug, Clone, PartialEq)]
enum ValueImpl<FieldName> {
    /// An array.
    Array(Vec<ValueImpl<FieldName>>),

    /// A dictionary.
    Dictionary(Vec<(ValueImpl<FieldName>, ValueImpl<FieldName>)>),

    /// A boolean value.
    Boolean(bool),

    /// A 32-bit signed integer.
    Int32(i32),

    /// A 64-bit signed integer.
    Int64(i64),

    /// An unsigned 32-bit integer.
    Uint32(u32),

    /// An unsigned 64-bit integer.
    Uint64(u64),

    /// A 32-bit floating point number.
    Float32(f32),

    /// A 64-bit floating point number.
    Float64(f64),

    /// A string.
    String(String),

    /// An enum.
    Enum(FieldName),

    /// A UUID.
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
}

impl<FieldName: Ord + Display> ValueImpl<FieldName> {
    /// Format the value as a string.
    fn fmt_for<Id>(
        &self,
        instance: &Arc<TypeDefinitionInstance<Id, FieldName>>,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match (self, &instance.attributes) {
            (Self::Array(items), TypeAttributesInstance::Array(a)) => {
                f.write_char('[')?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    item.fmt_for(a.items_type_id(), f)?;
                }
                f.write_char(']')?;
            }
            (Self::Dictionary(items), TypeAttributesInstance::Dictionary(a)) => {
                f.write_char('{')?;
                for (i, (key, value)) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    key.fmt_for(a.keys_type_id(), f)?;
                    f.write_str(": ")?;
                    value.fmt_for(a.values_type_id(), f)?;
                }
                f.write_char('}')?;
            }
            (Self::Boolean(v), TypeAttributesInstance::Boolean(_)) => write!(f, "{v}")?,
            (Self::Int32(v), TypeAttributesInstance::Int32(_)) => write!(f, "{v}")?,
            (Self::Int64(v), TypeAttributesInstance::Int64(_)) => write!(f, "{v}")?,
            (Self::Uint32(v), TypeAttributesInstance::Uint32(_)) => write!(f, "{v}")?,
            (Self::Uint64(v), TypeAttributesInstance::Uint64(_)) => write!(f, "{v}")?,
            (Self::Float32(v), TypeAttributesInstance::Float32(_)) => write!(f, "{v}")?,
            (Self::Float64(v), TypeAttributesInstance::Float64(_)) => write!(f, "{v}")?,
            (Self::String(v), TypeAttributesInstance::String(_)) => {
                f.write_char('"')?;
                f.write_str(v)?;
                f.write_char('"')?;
            }
            (Self::Enum(v), TypeAttributesInstance::Enum(_)) => {
                write!(f, "{}::{v}", instance.name)?
            }
            #[cfg(feature = "uuid")]
            (Self::Uuid(v), TypeAttributesInstance::Uuid(_)) => write!(f, "\"{v}\"")?,
            _ => {
                panic!("inconsistent value and type attributes");
            }
        }

        Ok(())
    }
}

/// An error that can occur when parsing a GameSON value implementation.
#[derive(Debug, thiserror::Error)]
enum ParseImplError {
    /// The dictionary key is invalid.
    #[error("invalid dictionary key: {0}")]
    InvalidDictionaryKey(#[source] Box<Self>),

    /// The dictionary value is invalid.
    #[error("invalid dictionary value: {0}")]
    InvalidDictionaryValue(#[source] Box<Self>),

    /// The number is invalid.
    #[error("invalid int32: {0}")]
    InvalidInt32(#[from] ValidateNumberTypeError<i32>),
}

impl<FieldName: Ord> ValueImpl<FieldName> {
    /// Parse a GameSON value for a specified type instance.
    fn parse_for<Id>(
        path: &mut ParseErrorPath,
        instance: &Arc<TypeDefinitionInstance<Id, FieldName>>,
        value: serde_json::Value,
    ) -> Result<Self, ParseImplError> {
        match (&instance.attributes, value) {
            (TypeAttributesInstance::Array(a), serde_json::Value::Array(v)) => {
                let items = v
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| {
                        path.push(ParseErrorPathSegment::ArrayIndex(i));
                        Self::parse_for(path, a.items_type_id(), v).map(|value| {
                            // We only must pop if the parse was successful.
                            path.pop();

                            value
                        })
                    })
                    .collect::<Result<Vec<Self>, _>>()?;

                Ok(Self::Array(items))
            }
            (TypeAttributesInstance::Dictionary(a), serde_json::Value::Object(v)) => {
                let items = v
                    .into_iter()
                    .map(|(k, v)| {
                        path.push(ParseErrorPathSegment::DictionaryKey(k.clone()));

                        let key =
                            Self::parse_for(path, a.keys_type_id(), serde_json::Value::String(k))
                                .map_err(Box::new)
                                .map_err(ParseImplError::InvalidDictionaryKey)?;

                        let value = Self::parse_for(path, a.values_type_id(), v)
                            .map_err(Box::new)
                            .map_err(ParseImplError::InvalidDictionaryValue)?;

                        // We only must pop if the parse was successful.
                        path.pop();

                        Result::<_, ParseImplError>::Ok((key, value))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Self::Dictionary(items))
            }
            (TypeAttributesInstance::Boolean(_), serde_json::Value::Bool(v)) => {
                Ok(Self::Boolean(v))
            }
            (TypeAttributesInstance::Int32(a), serde_json::Value::Number(v)) => {
                let v = v
                    .as_i64()
                    .ok_or(ValidateNumberTypeError::InvalidValue)?
                    .try_into()
                    .expect("failed to convert i64 to i32");

                a.validate(v)?;

                Ok(Self::Int32(v))
            }
            _ => unimplemented!(),
        }
    }
}
