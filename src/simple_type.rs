use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::EnumType;

/// A simple type in the GameSON format.
///
/// Simple types are the most basic types in GameSON. They cannot reference other types and are as
/// such, not parametrized by an identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleType<EnumName: Ord + Display + Clone> {
    /// A boolean value.
    Boolean {
        /// The default value of the boolean.
        default: Option<bool>,
    },

    /// A 32-bit signed integer.
    Int32 {
        /// The default value of the integer.
        default: Option<i32>,
    },

    /// A 64-bit signed integer.
    Int64 {
        /// The default value of the integer.
        default: Option<i64>,
    },

    /// An unsigned 32-bit integer.
    Uint32 {
        /// The default value of the integer.
        default: Option<u32>,
    },

    /// An unsigned 64-bit integer.
    Uint64 {
        /// The default value of the integer.
        default: Option<u64>,
    },

    /// A 32-bit floating point number.
    Float32 {
        /// The default value of the float.
        default: Option<f32>,
    },

    /// A 64-bit floating point number.
    Float64 {
        /// The default value of the float.
        default: Option<f64>,
    },

    /// A string value.
    String {
        /// The default value of the string.
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
        default: Option<uuid::Uuid>,
    },
}

impl<EnumName: Ord + Display + Clone> SimpleType<EnumName> {
    /// Returns whether the type has a default value.
    pub fn has_default(&self) -> bool {
        match self {
            SimpleType::Boolean { default } => default.is_some(),
            SimpleType::Int32 { default } => default.is_some(),
            SimpleType::Int64 { default } => default.is_some(),
            SimpleType::Uint32 { default } => default.is_some(),
            SimpleType::Uint64 { default } => default.is_some(),
            SimpleType::Float32 { default } => default.is_some(),
            SimpleType::Float64 { default } => default.is_some(),
            SimpleType::String { default } => default.is_some(),
            SimpleType::Enum(r#enum) => r#enum.has_default(),
            #[cfg(feature = "uuid")]
            SimpleType::Uuid { default } => default.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_has_default() {
        type SimpleType = super::SimpleType<String>;

        let t = SimpleType::Boolean { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Boolean {
            default: Some(true),
        };
        assert!(t.has_default());

        let t = SimpleType::Int32 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Int32 { default: Some(42) };
        assert!(t.has_default());

        let t = SimpleType::Int64 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Int64 { default: Some(42) };
        assert!(t.has_default());

        let t = SimpleType::Uint32 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Uint32 { default: Some(42) };
        assert!(t.has_default());

        let t = SimpleType::Uint64 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Uint64 { default: Some(42) };
        assert!(t.has_default());

        let t = SimpleType::Float32 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Float32 {
            default: Some(42.0),
        };
        assert!(t.has_default());

        let t = SimpleType::Float64 { default: None };
        assert!(!t.has_default());

        let t = SimpleType::Float64 {
            default: Some(42.0),
        };
        assert!(t.has_default());

        let t = SimpleType::String { default: None };
        assert!(!t.has_default());

        let t = SimpleType::String {
            default: Some("Hello".to_string()),
        };
        assert!(t.has_default());

        #[cfg(feature = "uuid")]
        {
            let t = SimpleType::Uuid { default: None };
            assert!(!t.has_default());

            let t = SimpleType::Uuid {
                default: Some(uuid::Uuid::default()),
            };
            assert!(t.has_default());
        }
    }
}
