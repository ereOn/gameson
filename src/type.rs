use std::fmt::Display;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::EnumType;

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
    ///
    /// Array types always have a default value of an empty array.
    Array {
        /// The items type identifier.
        items_type_id: Id,
    },

    /// A dictionary of key-value pairs.
    ///
    /// All the keys in a dictionary are of the same type, and all the values are of the same type.
    ///
    /// Dictionary types always have a default value of an empty dictionary.
    Dictionary {
        /// The keys type identifier.
        keys_type_id: Id,

        /// The values type identifier.
        values_type_id: Id,
    },

    /// A boolean value.
    Boolean {
        /// The default value of the boolean.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<bool>,
    },

    /// A 32-bit signed integer.
    Int32 {
        /// The default value of the integer.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<i32>,
    },

    /// A 64-bit signed integer.
    Int64 {
        /// The default value of the integer.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<i64>,
    },

    /// An unsigned 32-bit integer.
    Uint32 {
        /// The default value of the integer.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<u32>,
    },

    /// An unsigned 64-bit integer.
    Uint64 {
        /// The default value of the integer.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<u64>,
    },

    /// A 32-bit floating point number.
    Float32 {
        /// The default value of the float.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<f32>,
    },

    /// A 64-bit floating point number.
    Float64 {
        /// The default value of the float.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<f64>,
    },

    /// A string value.
    String {
        /// The default value of the string.
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String>,
    },

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

impl<Id, EnumName: Ord + Display + Clone + Serialize + DeserializeOwned> Type<Id, EnumName> {
    /// Returns whether the type has a default value.
    pub fn has_default(&self) -> bool {
        match self {
            Type::Array { .. } | Type::Dictionary { .. } => true,
            Type::Boolean { default } => default.is_some(),
            Type::Int32 { default } => default.is_some(),
            Type::Int64 { default } => default.is_some(),
            Type::Uint32 { default } => default.is_some(),
            Type::Uint64 { default } => default.is_some(),
            Type::Float32 { default } => default.is_some(),
            Type::Float64 { default } => default.is_some(),
            Type::String { default } => default.is_some(),
            Type::Enum(r#enum) => r#enum.has_default(),
            #[cfg(feature = "uuid")]
            Type::Uuid { default } => default.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    type Type = super::Type<u32, String>;

    #[test]
    fn test_has_default() {
        let t = Type::Array { items_type_id: 1 };
        assert!(t.has_default());

        let t = Type::Dictionary {
            keys_type_id: 1,
            values_type_id: 2,
        };
        assert!(t.has_default());

        let t = Type::Boolean { default: None };
        assert!(!t.has_default());

        let t = Type::Boolean {
            default: Some(true),
        };
        assert!(t.has_default());

        let t = Type::Int32 { default: None };
        assert!(!t.has_default());

        let t = Type::Int32 { default: Some(42) };
        assert!(t.has_default());

        let t = Type::Int64 { default: None };
        assert!(!t.has_default());

        let t = Type::Int64 { default: Some(42) };
        assert!(t.has_default());

        let t = Type::Uint32 { default: None };
        assert!(!t.has_default());

        let t = Type::Uint32 { default: Some(42) };
        assert!(t.has_default());

        let t = Type::Uint64 { default: None };
        assert!(!t.has_default());

        let t = Type::Uint64 { default: Some(42) };
        assert!(t.has_default());

        let t = Type::Float32 { default: None };
        assert!(!t.has_default());

        let t = Type::Float32 {
            default: Some(42.0),
        };
        assert!(t.has_default());

        let t = Type::Float64 { default: None };
        assert!(!t.has_default());

        let t = Type::Float64 {
            default: Some(42.0),
        };
        assert!(t.has_default());

        let t = Type::String { default: None };
        assert!(!t.has_default());

        let t = Type::String {
            default: Some("Hello".to_string()),
        };
        assert!(t.has_default());

        #[cfg(feature = "uuid")]
        {
            let t = Type::Uuid { default: None };
            assert!(!t.has_default());

            let t = Type::Uuid {
                default: Some(uuid::Uuid::default()),
            };
            assert!(t.has_default());
        }
    }

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

        let expected = Type::Boolean {
            default: Some(true),
        };

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "boolean",
                "attributes": {
                    "default": true
                }
            })
        );

        let t: Type = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);

        let expected = Type::Float32 { default: None };

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
