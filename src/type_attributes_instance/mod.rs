use std::{fmt::Display, sync::Arc};

use crate::{
    TypeDefinitionInstance,
    type_attributes::{
        ArrayTypeAttributes, BooleanTypeAttributes, DictionaryTypeAttributes, EnumTypeAttributes,
        NumberTypeAttributes, StringTypeAttributes,
    },
};

#[cfg(feature = "uuid")]
use crate::type_attributes::UuidTypeAttributes;

/// A type attributes instance.
#[derive(Debug)]
pub enum TypeAttributesInstance<Id, FieldName: Ord> {
    /// An array type.
    Array(ArrayTypeAttributes<Arc<TypeDefinitionInstance<Id, FieldName>>>),

    /// A dictionary type.
    Dictionary(DictionaryTypeAttributes<Arc<TypeDefinitionInstance<Id, FieldName>>>),

    /// A boolean type.
    Boolean(BooleanTypeAttributes),

    /// A 32-bit signed integer type.
    Int32(NumberTypeAttributes<i32>),

    /// A 64-bit signed integer type.
    Int64(NumberTypeAttributes<i64>),

    /// An unsigned 32-bit integer type.
    Uint32(NumberTypeAttributes<u32>),

    /// An unsigned 64-bit integer type.
    Uint64(NumberTypeAttributes<u64>),

    /// A 32-bit floating point number type.
    Float32(NumberTypeAttributes<f32>),

    /// A 64-bit floating point number type.
    Float64(NumberTypeAttributes<f64>),

    /// A string type.
    String(StringTypeAttributes),

    /// An enum type.
    Enum(EnumTypeAttributes<FieldName>),

    /// A UUID type.
    #[cfg(feature = "uuid")]
    Uuid(UuidTypeAttributes),
}

impl<Id, FieldName: Ord> Display for TypeAttributesInstance<Id, FieldName>
where
    Id: Display,
    FieldName: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(a) => write!(f, "array({a})"),
            Self::Dictionary(d) => write!(f, "dictionary({d})",),
            Self::Boolean(_) => f.write_str("boolean"),
            Self::Int32(n) => write!(f, "int32({n})"),
            Self::Int64(n) => write!(f, "int64({n})"),
            Self::Uint32(n) => write!(f, "uint32({n})"),
            Self::Uint64(n) => write!(f, "uint64({n})"),
            Self::Float32(n) => write!(f, "float32({n})"),
            Self::Float64(n) => write!(f, "float64({n})"),
            Self::String(s) => write!(f, "string({})", s),
            Self::Enum(e) => write!(f, "enum({})", e),
            #[cfg(feature = "uuid")]
            Self::Uuid(_) => f.write_str("uuid"),
        }
    }
}

impl<Id, FieldName: Ord> TypeAttributesInstance<Id, FieldName> {
    /// Check if the type is suitable for usage as a key in a dictionary.
    ///
    /// Usually, this means that the type serializes as a string.
    pub(crate) fn is_key_type(&self) -> bool {
        match self {
            Self::Array(_) => false,
            Self::Dictionary(_) => false,
            Self::Boolean(_) => false,
            Self::Int32(_) => false,
            Self::Int64(_) => false,
            Self::Uint32(_) => false,
            Self::Uint64(_) => false,
            Self::Float32(_) => false,
            Self::Float64(_) => false,
            Self::String(_) => true,
            Self::Enum(_) => true,
            #[cfg(feature = "uuid")]
            Self::Uuid(_) => true,
        }
    }
}
