use std::fmt::Display;

use crate::type_attributes_instance::TypeAttributesInstance;

/// A type instance.
///
/// This structure's purpose is to allow parse and validate GameSON values.
#[derive(Debug)]
pub struct TypeDefinitionInstance<Id, FieldName: Ord> {
    /// The identifier of the type.
    pub(crate) id: Id,

    /// The name of the type.
    pub(crate) name: FieldName,

    /// The type attributes.
    pub(crate) attributes: TypeAttributesInstance<Id, FieldName>,
}

impl<Id, FieldName> Display for TypeDefinitionInstance<Id, FieldName>
where
    Id: Display,
    FieldName: Ord + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            id,
            name,
            attributes,
        } = self;

        write!(f, "{name}({id}): {attributes}")
    }
}
