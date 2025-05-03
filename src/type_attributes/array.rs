use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::TypeDefinitionInstance;

/// Attributes for an array type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ArrayTypeAttributes<Id> {
    /// The items type identifier.
    items_type_id: Id,
}

impl<Id> ArrayTypeAttributes<Id> {
    /// Create new array type attributes.
    pub fn new(items_type_id: Id) -> Self {
        Self { items_type_id }
    }

    /// Get the items type identifier.
    pub fn items_type_id(&self) -> &Id {
        &self.items_type_id
    }
}

impl<Id: Display> Display for ArrayTypeAttributes<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { items_type_id } = self;

        items_type_id.fmt(f)
    }
}

impl<Id: Ord> ArrayTypeAttributes<Id> {
    /// Instantiate the array type attributes.
    ///
    /// The specified `refs_by_id` is used to resolve the type identifier of the items and must
    /// contain its id or the call will panic.
    pub(crate) fn instantiate<FieldName: Ord>(
        self,
        mut refs_by_id: BTreeMap<Id, Arc<TypeDefinitionInstance<Id, FieldName>>>,
    ) -> ArrayTypeAttributes<Arc<TypeDefinitionInstance<Id, FieldName>>> {
        ArrayTypeAttributes {
            items_type_id: refs_by_id
                .remove(&self.items_type_id)
                .expect("items_type_id not found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    type ArrayTypeAttributes = super::ArrayTypeAttributes<u32>;

    #[test]
    fn test_serialization() {
        let expected = ArrayTypeAttributes::new(1);

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "items_type_id": 1,
            })
        );

        let t: ArrayTypeAttributes = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
