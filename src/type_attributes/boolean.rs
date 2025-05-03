use serde::{Deserialize, Serialize};

/// Attributes for a boolean type.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct BooleanTypeAttributes {}

#[cfg(test)]
mod tests {
    use super::BooleanTypeAttributes;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let expected = BooleanTypeAttributes::default();

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(json, json!({}));

        let t: BooleanTypeAttributes = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
