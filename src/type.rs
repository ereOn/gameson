use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::SimpleType;

/// An generic enumeration of the different types of GameSON values.
///
/// An instance of ``Type`` is used to represent a type of value in a GameSON context.
///
/// # Generic parameters
///
/// * `Id`: The type of the identifier used in the GameSON format. This is typically a string, uuid
///   or an integer, depending on the specific implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// A simple type.
    Simple {
        /// The simple type identifier.
        inner: SimpleType<EnumName>,
    },
}

impl<Id, EnumName: Ord + Display + Clone> Type<Id, EnumName> {
    /// Returns whether the type has a default value.
    pub fn has_default(&self) -> bool {
        match self {
            Type::Array { .. } | Type::Dictionary { .. } => true,
            Type::Simple { inner } => inner.has_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SimpleType;

    #[test]
    fn test_has_default() {
        type Type = super::Type<u32, String>;

        let t = Type::Array { items_type_id: 1 };
        assert!(t.has_default());

        let t = Type::Dictionary {
            keys_type_id: 1,
            values_type_id: 2,
        };
        assert!(t.has_default());

        let t = Type::Simple {
            inner: SimpleType::Boolean { default: None },
        };
        assert!(!t.has_default());

        let t = Type::Simple {
            inner: SimpleType::Boolean {
                default: Some(true),
            },
        };
        assert!(t.has_default());
    }
}
