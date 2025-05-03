//! GameSON encoding format.

pub(crate) mod type_attributes;

mod type_definition;
mod typed_value;

pub use type_attributes::TypeAttributes;
pub use type_definition::TypeDefinition;
pub use typed_value::TypedValue;
