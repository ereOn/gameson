//! Type attributes.

mod array;
mod boolean;
mod dictionary;
mod r#enum;
mod number;
mod string;

#[cfg(feature = "uuid")]
mod uuid;

use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

pub(crate) use array::ArrayTypeAttributes;
pub(crate) use boolean::BooleanTypeAttributes;
pub(crate) use dictionary::DictionaryTypeAttributes;
pub(crate) use r#enum::EnumTypeAttributes;
pub(crate) use number::NumberTypeAttributes;
pub(crate) use string::StringTypeAttributes;

#[cfg(feature = "uuid")]
pub(crate) use uuid::UuidTypeAttributes;

use crate::{TypeDefinitionInstance, type_attributes_instance::TypeAttributesInstance};

/// All the different types and their attributes, supported by the GameSON format.
///
/// # Generic parameters
///
/// * `Id`: The type of the type identifier used in the GameSON format. This is typically a string,
///   uuid or an integer, depending on the specific implementation.
/// * `FieldName`: The type of the field name used in the GameSON format. This is typically a
///   string-like type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum TypeAttributes<Id, FieldName: Ord + Display + Clone> {
    /// An array of values of the same type.
    Array(ArrayTypeAttributes<Id>),

    /// A dictionary of key-value pairs.
    ///
    /// All the keys in a dictionary are of the same type, and all the values are of the same type.
    Dictionary(DictionaryTypeAttributes<Id>),

    /// A boolean value.
    Boolean(BooleanTypeAttributes),

    /// A 32-bit signed integer.
    Int32(NumberTypeAttributes<i32>),

    /// A 64-bit signed integer.
    Int64(NumberTypeAttributes<i64>),

    /// An unsigned 32-bit integer.
    Uint32(NumberTypeAttributes<u32>),

    /// An unsigned 64-bit integer.
    Uint64(NumberTypeAttributes<u64>),

    /// A 32-bit floating point number.
    Float32(NumberTypeAttributes<f32>),

    /// A 64-bit floating point number.
    Float64(NumberTypeAttributes<f64>),

    /// A string value.
    String(StringTypeAttributes),

    /// An enumeration value.
    ///
    /// An enum is a type that can take on a limited set of values. The values are defined by the
    /// type itself.
    Enum(EnumTypeAttributes<FieldName>),

    #[cfg(feature = "uuid")]
    /// An UUID value.
    Uuid(UuidTypeAttributes),
}

impl<Id, FieldName: Ord + Display + Clone> TypeAttributes<Id, FieldName> {
    /// Get the external identifier references of this type attributes.
    pub fn external_identifier_references(&self) -> Vec<&Id> {
        match self {
            TypeAttributes::Array(a) => vec![a.items_type_id()],
            TypeAttributes::Dictionary(d) => vec![d.keys_type_id(), d.values_type_id()],
            TypeAttributes::Boolean(_) => vec![],
            TypeAttributes::Int32(_) => vec![],
            TypeAttributes::Int64(_) => vec![],
            TypeAttributes::Uint32(_) => vec![],
            TypeAttributes::Uint64(_) => vec![],
            TypeAttributes::Float32(_) => vec![],
            TypeAttributes::Float64(_) => vec![],
            TypeAttributes::String(_) => vec![],
            TypeAttributes::Enum(_) => vec![],
            #[cfg(feature = "uuid")]
            TypeAttributes::Uuid(_) => vec![],
        }
    }
}

/// A result for an instantation of type attributes.
pub type InstantiationResult<T, Id, FieldName> = Result<T, InstantiationError<Id, FieldName>>;

/// An error that can occur when instantiating type attributes.
#[derive(Debug, thiserror::Error)]
pub enum InstantiationError<Id, FieldName> {
    /// The dictionary key type is not appropriate.
    #[error(
        "cannot use type `{key_type_id}` (`{key_type_name}`) of type `{key_type_str}` as key type for dictionary type"
    )]
    InappropriateKeyType {
        key_type_id: Id,
        key_type_name: FieldName,
        key_type_str: String,
    },
}

impl<Id: Ord + Clone + Display, FieldName: Ord + Clone + Display> TypeAttributes<Id, FieldName> {
    /// Instantiates the type attributes.
    ///
    /// This function resolves the type identifiers of the type attributes to their actual type
    /// attributes instances.
    ///
    /// # Panics
    ///
    /// This function panics if the necessary type identifiers are not found in the `refs_by_id` map.
    pub(crate) fn instantiate(
        self,
        refs_by_id: BTreeMap<Id, Arc<TypeDefinitionInstance<Id, FieldName>>>,
    ) -> Result<TypeAttributesInstance<Id, FieldName>, InstantiationError<Id, FieldName>> {
        Ok(match self {
            TypeAttributes::Array(a) => TypeAttributesInstance::Array(a.instantiate(refs_by_id)),
            TypeAttributes::Dictionary(d) => {
                TypeAttributesInstance::Dictionary(d.instantiate(refs_by_id)?)
            }
            TypeAttributes::Boolean(b) => TypeAttributesInstance::Boolean(b),
            TypeAttributes::Int32(i) => TypeAttributesInstance::Int32(i),
            TypeAttributes::Int64(i) => TypeAttributesInstance::Int64(i),
            TypeAttributes::Uint32(i) => TypeAttributesInstance::Uint32(i),
            TypeAttributes::Uint64(i) => TypeAttributesInstance::Uint64(i),
            TypeAttributes::Float32(f) => TypeAttributesInstance::Float32(f),
            TypeAttributes::Float64(f) => TypeAttributesInstance::Float64(f),
            TypeAttributes::String(s) => TypeAttributesInstance::String(s),
            TypeAttributes::Enum(e) => TypeAttributesInstance::Enum(e),
            #[cfg(feature = "uuid")]
            TypeAttributes::Uuid(u) => TypeAttributesInstance::Uuid(u),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::type_attributes::{
        array::ArrayTypeAttributes, boolean::BooleanTypeAttributes,
        dictionary::DictionaryTypeAttributes,
    };

    use super::NumberTypeAttributes;

    type Type = super::TypeAttributes<u32, String>;

    #[test]
    fn test_serialization() {
        let expected = Type::Array(ArrayTypeAttributes::new(1));

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "array",
                "attributes": {
                    "items_type_id": 1
                }
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);

        let expected = Type::Dictionary(DictionaryTypeAttributes::new(1, 2));

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "dictionary",
                "attributes": {
                    "keys_type_id": 1,
                    "values_type_id": 2
                }
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);

        let expected = Type::Boolean(BooleanTypeAttributes::default());

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "boolean",
                "attributes": {}
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);

        let expected = Type::Int32(
            NumberTypeAttributes::builder()
                .min(0)
                .max(10)
                .build()
                .unwrap(),
        );

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "int32",
                "attributes": {
                    "min": 0,
                    "max": 10,
                }
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
        let expected = Type::Float32(NumberTypeAttributes::default());

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "float32",
                "attributes": {}
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
