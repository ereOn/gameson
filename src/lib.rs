//! GameSON encoding format.

pub(crate) mod type_attributes;
pub(crate) mod type_attributes_instance;

mod type_definition;
mod type_definition_instance;
mod type_definition_registry;
mod typed_value;

pub use type_attributes::{InstantiationError, InstantiationResult, TypeAttributes};
pub use type_definition::TypeDefinition;
pub use type_definition_instance::TypeDefinitionInstance;
pub use type_definition_registry::TypeDefinitionRegistry;
pub use typed_value::TypedValue;
