use serde::{Deserialize, Serialize};

/// Attributes for a UUID type.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct UuidTypeAttributes {}

#[cfg(test)]
mod tests {
    use super::UuidTypeAttributes;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let expected = UuidTypeAttributes::default();

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(json, json!({}));

        let t: UuidTypeAttributes = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
