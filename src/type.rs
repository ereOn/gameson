use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{EnumType, NumberType};

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
pub enum Type<Id, EnumName: Ord + Display + Clone> {
    /// An array of values of the same type.
    Array {
        /// The items type identifier.
        items_type_id: Id,
    },

    /// A dictionary of key-value pairs.
    ///
    /// All the keys in a dictionary are of the same type, and all the values are of the same type.
    Dictionary {
        /// The keys type identifier.
        keys_type_id: Id,

        /// The values type identifier.
        values_type_id: Id,
    },

    /// A boolean value.
    Boolean {},

    /// A 32-bit signed integer.
    Int32(NumberType<i32>),

    /// A 64-bit signed integer.
    Int64(NumberType<i64>),

    /// An unsigned 32-bit integer.
    Uint32(NumberType<u32>),

    /// An unsigned 64-bit integer.
    Uint64(NumberType<u64>),

    /// A 32-bit floating point number.
    Float32(NumberType<f32>),

    /// A 64-bit floating point number.
    Float64(NumberType<f64>),

    /// A string value.
    String {},

    /// An enumeration value.
    ///
    /// An enum is a type that can take on a limited set of values. The values are defined by the
    /// type itself.
    Enum(EnumType<EnumName>),

    #[cfg(feature = "uuid")]
    /// An UUID value.
    Uuid {
        /// The default value of the UUID.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<uuid::Uuid>,
    },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::NumberType;

    type Type = super::Type<u32, String>;

    #[test]
    fn test_serialization() {
        let expected = Type::Array { items_type_id: 1 };

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

        let expected = Type::Dictionary {
            keys_type_id: 1,
            values_type_id: 2,
        };

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

        let expected = Type::Boolean {};

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

        let expected = Type::Int32(NumberType::builder().min(0).max(10).build().unwrap());

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
        let expected = Type::Float32(NumberType::default());

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
