use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Attributes for a number type.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct NumberTypeAttributes<Num> {
    /// The minimum value of the number.
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<Num>,

    /// The maximum value of the number.
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<Num>,
}

impl<Num: Display> Display for NumberTypeAttributes<Num> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { min, max } = self;
        match (min, max) {
            (Some(min), Some(max)) => write!(f, "{min}..{max}"),
            (Some(min), None) => write!(f, "{min}.."),
            (None, Some(max)) => write!(f, "..{max}"),
            (None, None) => f.write_str(".."),
        }
    }
}

impl<'de, Num: Copy + Display + PartialOrd + Deserialize<'de>> Deserialize<'de>
    for NumberTypeAttributes<Num>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct X<T> {
            #[serde(skip_serializing_if = "Option::is_none")]
            min: Option<T>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max: Option<T>,
        }

        let x = X::deserialize(deserializer)?;

        NumberTypeAttributes::new(x.min, x.max)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

/// An error that can occur when instantiating int type attributes.
#[derive(Debug, thiserror::Error)]
pub enum NewNumberTypeAttributesError<Num> {
    /// The range is invalid.
    #[error("invalid range: {0} > {1}")]
    InvalidRange(Num, Num),
}

impl<Num: PartialOrd + Copy> NumberTypeAttributes<Num> {
    /// Create a builder for the number type.
    pub fn builder() -> NumberTypeAttributesBuilder<Num> {
        NumberTypeAttributesBuilder::default()
    }

    /// Creates a new number type.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The range is invalid.
    fn new(min: Option<Num>, max: Option<Num>) -> Result<Self, NewNumberTypeAttributesError<Num>> {
        if let (Some(min), Some(max)) = (min, max) {
            if min > max {
                return Err(NewNumberTypeAttributesError::InvalidRange(min, max));
            }
        }

        Ok(Self { min, max })
    }
}

/// A builder for number type attributes.
#[derive(Debug)]
pub struct NumberTypeAttributesBuilder<Num> {
    min: Option<Num>,
    max: Option<Num>,
}

impl<Num> Default for NumberTypeAttributesBuilder<Num> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

impl<Num: PartialOrd + Copy> NumberTypeAttributesBuilder<Num> {
    /// Sets the minimum value of the number.
    pub fn min(mut self, min: Num) -> Self {
        self.min = Some(min);
        self
    }

    /// Sets the maximum value of the number.
    pub fn max(mut self, max: Num) -> Self {
        self.max = Some(max);
        self
    }

    /// Builds the number type.
    pub fn build(self) -> Result<NumberTypeAttributes<Num>, NewNumberTypeAttributesError<Num>> {
        NumberTypeAttributes::new(self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    type NumberType = super::NumberTypeAttributes<u32>;

    #[test]
    fn test_serialization() {
        let expected = NumberType::builder().min(0).max(10).build().unwrap();

        let json = serde_json::to_value(&expected).unwrap();
        assert_eq!(
            json,
            json!({
                "min": 0,
                "max": 10
            })
        );

        let t: NumberType = serde_json::from_value(json).unwrap();
        assert_eq!(t, expected);
    }
}
