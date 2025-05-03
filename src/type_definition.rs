use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::TypeAttributes;

/// A type definition for a GameSON type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TypeDefinition<Id, FieldName: Ord + Display + Clone> {
    /// The identifier of the type.
    pub id: Id,

    /// A description for the type.
    pub description: Option<String>,

    /// The type.
    #[serde(flatten)]
    pub attributes: TypeAttributes<Id, FieldName>,
}
