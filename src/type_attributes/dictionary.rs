use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::TypeDefinitionInstance;

use super::{InstantiationError, InstantiationResult};

/// Attributes for a dictionary type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct DictionaryTypeAttributes<Id> {
    /// The keys type identifier.
    keys_type_id: Id,

    /// The values type identifier.
    values_type_id: Id,
}

impl<Id> DictionaryTypeAttributes<Id> {
    /// Create new array type attributes.
    pub fn new(keys_type_id: Id, values_type_id: Id) -> Self {
        Self {
            keys_type_id,
            values_type_id,
        }
    }

    /// Get the keys type identifier.
    pub fn keys_type_id(&self) -> &Id {
        &self.keys_type_id
    }

    /// Get the values type identifier.
    pub fn values_type_id(&self) -> &Id {
        &self.values_type_id
    }
}

impl<Id: Display> Display for DictionaryTypeAttributes<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            keys_type_id,
            values_type_id,
        } = self;

        write!(f, "({keys_type_id}, {values_type_id})")
    }
}

impl<Id: Ord + Clone + Display> DictionaryTypeAttributes<Id> {
    /// Instantiate the array type attributes.
    ///
    /// The specified `refs_by_id` is used to resolve the type identifier of the items and must
    /// contain its id or the call will panic.
    pub(crate) fn instantiate<FieldName: Ord + Clone + Display>(
        &self,
        mut refs_by_id: BTreeMap<Id, Arc<TypeDefinitionInstance<Id, FieldName>>>,
    ) -> InstantiationResult<
        DictionaryTypeAttributes<Arc<TypeDefinitionInstance<Id, FieldName>>>,
        Id,
        FieldName,
    > {
        let keys_type_id = refs_by_id
            .remove(&self.keys_type_id)
            .expect("keys_type_id not found");

        if !keys_type_id.attributes.is_key_type() {
            return Err(InstantiationError::InappropriateKeyType {
                key_type_id: keys_type_id.id.clone(),
                key_type_name: keys_type_id.name.clone(),
                key_type_str: keys_type_id.attributes.to_string(),
            });
        }

        let values_type_id = refs_by_id
            .remove(&self.values_type_id)
            .expect("values_type_id not found");

        Ok(DictionaryTypeAttributes {
            keys_type_id,
            values_type_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    type DictionaryTypeAttributes = super::DictionaryTypeAttributes<u32>;

    #[test]
    fn test_serialization() {
        let expected = DictionaryTypeAttributes::new(1, 2);

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "keys_type_id": 1,
                "values_type_id": 2,
            })
        );

        let t: DictionaryTypeAttributes = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
