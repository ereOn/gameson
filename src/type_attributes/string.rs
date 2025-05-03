use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// A number type.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct StringTypeAttributes {}

impl Display for StringTypeAttributes {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {} = self;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::StringTypeAttributes;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let expected = StringTypeAttributes::default();

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(json, json!({}));

        let t: StringTypeAttributes = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
