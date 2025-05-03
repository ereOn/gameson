use serde::{Deserialize, Serialize};

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
