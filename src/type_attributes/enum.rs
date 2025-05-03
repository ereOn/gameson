use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

/// Attributes for an enum type.
///
/// Enumeration types allow for a limited set of values, which are defined by the type itself.
///
/// An enum type can always be expanded by adding new values to its definition. It is however never
/// allowed to remove values from an enum type definition. The idea is that if some shipped piece
/// of code was using a particular enum value, we should still be able to able to parse it with an
/// updated version of the type.
///
/// To circumvent this constraint, it is however possible to deprecate an enum value. Parsing a
/// deprecated enum value will work as it would with a normal enum value, but a warning will be
/// emitted for it.
///
/// Additionally, enum values can have aliases that allow for several enum names to represent the
/// same variant.
///
/// Aliases can never overlap with other enum names.
///
/// Empty enum types are allowed, although no value will satisfy their parsing requirements.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct EnumTypeAttributes<EnumName> {
    /// The values of the enum.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    values: BTreeMap<EnumName, EnumTypeValue>,

    /// The aliases of the enum.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    aliases: BTreeMap<EnumName, EnumName>,

    /// A default value for the enum.
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<EnumName>,
}

impl<EnumName> Default for EnumTypeAttributes<EnumName> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            aliases: Default::default(),
            default: Default::default(),
        }
    }
}

impl<EnumName> EnumTypeAttributes<EnumName> {
    /// Return a builder for the enum type.
    pub fn builder() -> EnumTypeAttributesBuilder<EnumName> {
        EnumTypeAttributesBuilder::default()
    }
}

/// An error that can occur when instantiating enum type attributes.
#[derive(Debug, thiserror::Error)]
pub enum NewEnumTypeAttributesError<EnumName> {
    /// An enum value is also an alias.
    #[error("enum value `{0}` is also an alias")]
    EnumValueIsAlias(EnumName),

    /// An enum alias points to a non-existant value.
    #[error("enum alias `{0}` points to a non-existant value `{1}`")]
    EnumAliasPointsToNonExistantValue(EnumName, EnumName),

    /// The default value is not a valid enum value.
    #[error("default value `{0}` is not a valid enum value")]
    DefaultValueIsNotAValidEnumValue(EnumName),
}

impl<EnumName: Ord + Display + Clone> EnumTypeAttributes<EnumName> {
    /// Creates a new enum type.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - An enum value is also an alias.
    /// - An enum alias points to a non-existant value.
    /// - The default value is not a valid enum value.
    fn new(
        values: BTreeMap<EnumName, EnumTypeValue>,
        aliases: BTreeMap<EnumName, EnumName>,
        default: Option<EnumName>,
    ) -> Result<Self, NewEnumTypeAttributesError<EnumName>> {
        for (alias, value) in &aliases {
            if values.contains_key(alias) {
                return Err(NewEnumTypeAttributesError::EnumValueIsAlias(alias.clone()));
            }

            if !values.contains_key(value) {
                return Err(
                    NewEnumTypeAttributesError::EnumAliasPointsToNonExistantValue(
                        alias.clone(),
                        value.clone(),
                    ),
                );
            }
        }

        if let Some(default) = &default {
            if !values.contains_key(default) {
                return Err(
                    NewEnumTypeAttributesError::DefaultValueIsNotAValidEnumValue(default.clone()),
                );
            }
        }

        Ok(Self {
            values,
            aliases,
            default,
        })
    }
}

impl<'de, EnumName: Ord + Display + Clone + Deserialize<'de>> Deserialize<'de>
    for EnumTypeAttributes<EnumName>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct X<T: Ord> {
            #[serde(default = "BTreeMap::new")]
            values: BTreeMap<T, EnumTypeValue>,
            #[serde(default = "BTreeMap::new")]
            aliases: BTreeMap<T, T>,
            default: Option<T>,
        }

        let x = X::deserialize(deserializer)?;

        Self::new(x.values, x.aliases, x.default)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

/// An enumeration type value.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
struct EnumTypeValue {
    /// A description for the enum type value.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Whether the enum value is deprecated.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    deprecated: bool,
}

/// A builder for enum type attributes.
#[derive(Debug)]
pub struct EnumTypeAttributesBuilder<EnumName> {
    /// The values of the enum.
    values: BTreeMap<EnumName, EnumTypeValue>,

    /// The aliases of the enum.
    aliases: BTreeMap<EnumName, EnumName>,

    /// A default value for the enum.
    default: Option<EnumName>,
}

impl<EnumName> Default for EnumTypeAttributesBuilder<EnumName> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            aliases: Default::default(),
            default: Default::default(),
        }
    }
}

impl<EnumName: Ord + Display + Clone> EnumTypeAttributesBuilder<EnumName> {
    /// Add a value to the enum type.
    pub fn with_value(mut self, name: EnumName) -> Self {
        self.values.insert(name, EnumTypeValue::default());
        self
    }
    /// Add a value to the enum type, with an optional description and deprecation status.
    pub fn with_value_ext(
        mut self,
        name: EnumName,
        description: Option<String>,
        deprecated: bool,
    ) -> Self {
        self.values.insert(
            name,
            EnumTypeValue {
                description,
                deprecated,
            },
        );

        self
    }

    /// Add an alias to the enum type.
    pub fn with_alias(mut self, name: EnumName, value: EnumName) -> Self {
        self.aliases.insert(name, value);
        self
    }

    /// Set the default value for the enum type.
    pub fn with_default(mut self, name: EnumName) -> Self {
        self.default = Some(name);
        self
    }

    /// Builds the enum type.
    pub fn build(
        self,
    ) -> Result<EnumTypeAttributes<EnumName>, NewEnumTypeAttributesError<EnumName>> {
        EnumTypeAttributes::new(self.values, self.aliases, self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::EnumTypeValue;
    use serde_json::json;

    type EnumTypeAttributes = super::EnumTypeAttributes<&'static str>;
    type NewEnumTypeAttributesError = super::NewEnumTypeAttributesError<&'static str>;

    #[test]
    fn test_validation() {
        EnumTypeAttributes::new(Default::default(), Default::default(), None).unwrap();

        assert!(matches!(
            EnumTypeAttributes::new(Default::default(), Default::default(), Some("foo"),)
                .unwrap_err(),
            NewEnumTypeAttributesError::DefaultValueIsNotAValidEnumValue("foo")
        ));

        EnumTypeAttributes::new(
            [(
                "foo",
                EnumTypeValue {
                    description: None,
                    deprecated: false,
                },
            )]
            .into_iter()
            .collect(),
            [("bar", "foo")].into_iter().collect(),
            None,
        )
        .unwrap();

        assert!(matches!(
            EnumTypeAttributes::new(
                [(
                    "foo",
                    EnumTypeValue {
                        description: None,
                        deprecated: false
                    }
                )]
                .into_iter()
                .collect(),
                [("foo", "bar")].into_iter().collect(),
                None,
            )
            .unwrap_err(),
            NewEnumTypeAttributesError::EnumValueIsAlias("foo")
        ));

        assert!(matches!(
            EnumTypeAttributes::new(
                [(
                    "foo",
                    EnumTypeValue {
                        description: None,
                        deprecated: false
                    }
                )]
                .into_iter()
                .collect(),
                [("bar", "zoo")].into_iter().collect(),
                None,
            )
            .unwrap_err(),
            NewEnumTypeAttributesError::EnumAliasPointsToNonExistantValue("bar", "zoo")
        ));
    }

    #[test]
    fn test_serialization() {
        type EnumType = super::EnumTypeAttributes<String>;

        let expected = EnumType::new(
            [(
                "foo".to_owned(),
                EnumTypeValue {
                    description: None,
                    deprecated: false,
                },
            )]
            .into_iter()
            .collect(),
            [("bar".to_owned(), "foo".to_owned())].into_iter().collect(),
            Some("foo".to_owned()),
        )
        .unwrap();

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "values": {
                    "foo": {},
                },
                "aliases": {
                    "bar": "foo",
                },
                "default": "foo",
            })
        );

        let t: EnumType = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
