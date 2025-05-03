//! Type attributes.

mod array;
mod boolean;
mod dictionary;
mod r#enum;
mod number;
mod string;

#[cfg(feature = "uuid")]
mod uuid;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use array::ArrayTypeAttributes;
use boolean::BooleanTypeAttributes;
use dictionary::DictionaryTypeAttributes;
use r#enum::EnumTypeAttributes;
use number::NumberTypeAttributes;
use string::StringTypeAttributes;

#[cfg(feature = "uuid")]
use uuid::UuidTypeAttributes;

/// An generic enumeration of the different types of GameSON values.
///
/// An instance of ``Type`` is used to represent a type of value in a GameSON context.
///
/// # Generic parameters
///
/// * `Id`: The type of the identifier used in the GameSON format. This is typically a string, uuid
///   or an integer, depending on the specific implementation.
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
