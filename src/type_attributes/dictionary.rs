use serde::{Deserialize, Serialize};

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
