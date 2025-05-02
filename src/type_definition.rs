use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Type;

/// A type definition for a GameSON type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TypeDefinition<Id, EnumName: Ord + Display + Clone> {
    /// The identifier of the type.
    pub id: Id,

    /// A description for the type.
    pub description: Option<String>,

    /// The type.
    pub r#type: Type<Id, EnumName>,
}
