use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::TypeAttributes;

/// A type definition for a GameSON type.
///
/// This structure's purpose is to store/retrieve type definitions either to disk or in a database.
///
/// Type definitions reference other type definitions by their identifiers, which can lead to
/// broken or circular references. In order to validate the integrity of the type definitions
/// hierarchy, those must be loaded into a
/// [`TypeDefinitionRegistry`](crate::TypeDefinitionRegistry).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TypeDefinition<Id, FieldName: Ord + Display + Clone> {
    /// The identifier of the type.
    ///
    /// Identifiers must be unique for different types.
    pub id: Id,

    /// A name for the type.
    ///
    /// Names must be unique for different types.
    pub name: FieldName,

    /// A description for the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The type.
    #[serde(flatten)]
    pub attributes: TypeAttributes<Id, FieldName>,
}
