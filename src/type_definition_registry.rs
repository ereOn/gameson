//! A registry of type definitions.

use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    sync::Arc,
};

use crate::{InstantiationError, TypeDefinition, TypeDefinitionInstance};

/// A registry of type definitions.
#[derive(Debug, Clone, Default)]
pub struct TypeDefinitionRegistry<Id, FieldName: Ord + Display + Clone> {
    /// The type definitions instances, by their identifiers.
    by_id: BTreeMap<Id, Arc<TypeDefinitionInstance<Id, FieldName>>>,

    /// The type definitions, by their names.
    by_name: BTreeMap<FieldName, Arc<TypeDefinitionInstance<Id, FieldName>>>,
}

/// An error that can occur when registering type definitions.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistrationError<Id, FieldName> {
    /// A type definition with the same identifier already exists.
    #[error("another type definition `{existing_name}` with the same id already exists")]
    DuplicateTypeDefinition { existing_name: FieldName },

    /// A type definition with the same name already exists.
    #[error("another type definition with id `{existing_id}` has the same name")]
    DuplicateTypeDefinitionName { existing_id: Id },

    /// A type definition has a broken reference.
    #[error("type definition has a broken reference to type definition `{referenced_id}`")]
    BrokenReference { referenced_id: Id },

    /// A type definition has a circular reference.
    #[error(
        "registering type definition would cause a circular reference cycle: {}",
        cycle.iter().map(|(id, name)| format!("`{name}` (`{id}`)")).join(" -> ")
    )]
    CircularReference { cycle: Vec<(Id, FieldName)> },

    /// A type definition has a blocked reference.
    #[error("type definition has a reference to a type definition that cannot be registered")]
    BlockedReference,

    /// An error occurred while instantiating the type attributes.
    #[error("unable to instantiate type attributes for type definition: {0}")]
    InstantiationError(#[from] InstantiationError<Id, FieldName>),
}

impl<Id: Ord + Clone + Display, FieldName: Ord + Clone + Display>
    TypeDefinitionRegistry<Id, FieldName>
{
    /// Register type definitions.
    ///
    /// The passed-in type definitions can only have references to other, previously registered,
    /// type definitions or to other types definitions in the same batch.
    ///
    /// If the batch contains broken or circular references, those type definitions will not be
    /// registered.
    ///
    /// If the batch contains duplicate type definitions, those will not be registered.
    ///
    /// The method returns list of all the type definitions that were registered as well as those
    /// who were not registered alongside the reason why they were not registered.
    #[expect(
        clippy::type_complexity,
        reason = "inherent associated types are not yet stable so we can't do much about it here"
    )]
    pub fn register(
        &mut self,
        type_definitions: impl IntoIterator<Item = TypeDefinition<Id, FieldName>>,
    ) -> (
        Vec<Arc<TypeDefinitionInstance<Id, FieldName>>>,
        Vec<(
            TypeDefinition<Id, FieldName>,
            RegistrationError<Id, FieldName>,
        )>,
    ) {
        // This gives us a list of all the type definitions to register, with the references they
        // have.
        let mut type_definitions: Vec<_> = type_definitions
            .into_iter()
            .map(|td| {
                (
                    td.attributes
                        .external_identifier_references()
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>(),
                    td,
                )
            })
            .collect();

        // Contains the list of type definitions that have not been registered yet.
        let mut postponed_type_definitions = Vec::with_capacity(type_definitions.len());
        let mut last_count = type_definitions.len();
        let mut failed_type_definitions = Vec::new();
        let mut registered_type_definitions = Vec::new();

        // While we have type definitions to register, we continue.
        while !type_definitions.is_empty() {
            // By sorting the definitions by the ascending number of references, we can ensure that the
            // first type definitions to be registered are the ones with the least number of
            // references and the lesser likelihood of broken or circular references.
            type_definitions.sort_by_key(|(refs, _)| refs.len());

            'outer: for (refs, mut td) in type_definitions {
                // Check for duplicate type definitions.
                if let Some(existing) = self.by_id.get(&td.id) {
                    failed_type_definitions.push((
                        td,
                        RegistrationError::DuplicateTypeDefinition {
                            existing_name: existing.name.clone(),
                        },
                    ));

                    continue 'outer;
                }

                if let Some(existing) = self.by_name.get(&td.name) {
                    failed_type_definitions.push((
                        td,
                        RegistrationError::DuplicateTypeDefinitionName {
                            existing_id: existing.id.clone(),
                        },
                    ));

                    continue 'outer;
                }

                let mut refs_by_id = BTreeMap::new();

                for ref_ in &refs {
                    // Ensure that the reference was already registered.
                    match self.by_id.get(ref_) {
                        Some(inst) => {
                            refs_by_id.insert(ref_.clone(), Arc::clone(inst));
                        }
                        None => {
                            // If the reference was not registered, we need to postpone the
                            // registration of this type definition.
                            //
                            // This is not an error (yet), as we might be able to register it
                            // later.
                            postponed_type_definitions.push((refs, td));
                            continue 'outer;
                        }
                    }
                }

                // Instantiate the type attributes: this can fail if the type attributes are
                // incompatible (for instance if the key type of a dictionary is not a key-type).

                let attributes = match td.attributes.instantiate(refs_by_id) {
                    Ok(attributes) => attributes,
                    Err((attributes, err)) => {
                        td.attributes = attributes;

                        failed_type_definitions
                            .push((td, RegistrationError::InstantiationError(err)));

                        continue 'outer;
                    }
                };

                // At this point all the references were looked up and there are no duplicates: we
                // can register the type definition.
                let type_definition_instance = TypeDefinitionInstance {
                    id: td.id,
                    name: td.name,
                    attributes,
                };

                // Register the type definition.
                registered_type_definitions
                    .push(self.insert_type_definition_instance(type_definition_instance));
            }

            type_definitions = std::mem::take(&mut postponed_type_definitions);

            // If the number of type definitions to register did not change, we have a broken or circular
            // reference.
            if type_definitions.len() == last_count {
                // Compute a list of all remaining identifiers to register.
                let remaining_ids: BTreeSet<_> = type_definitions
                    .iter()
                    .map(|(_, td)| td.id.clone())
                    .collect();

                // Check for broken references.
                'outer: for (refs, td) in type_definitions {
                    for ref_ in &refs {
                        if !(remaining_ids.contains(ref_) || self.by_id.contains_key(ref_)) {
                            failed_type_definitions.push((
                                td,
                                RegistrationError::BrokenReference {
                                    referenced_id: ref_.clone(),
                                },
                            ));

                            continue 'outer;
                        }
                    }

                    postponed_type_definitions.push((refs, td));
                }

                type_definitions = std::mem::take(&mut postponed_type_definitions);

                // The remaining type definitions are the ones that lead to circular references.
                loop {
                    let deps = type_definitions
                        .iter()
                        .map(|(refs, td)| (td.id.clone(), refs.iter().cloned().collect()))
                        .collect::<BTreeMap<_, _>>();

                    let cycle = detect_minimal_cycle(&deps);

                    if cycle.is_empty() {
                        // No cycle found: we can break.
                        break;
                    }

                    let mut cyclic_type_definitions = Vec::with_capacity(cycle.len() - 1);

                    for (refs_, td) in std::mem::take(&mut type_definitions) {
                        if cycle.contains(&td.id) {
                            cyclic_type_definitions.push(td);
                        } else {
                            postponed_type_definitions.push((refs_, td));
                        }
                    }

                    let cycle = cycle
                        .into_iter()
                        .map(|id| {
                            // It's impossible for the cycle to contain an id that is not in the new
                            // type definitions, as the already registered type definitions are
                            // guaranteed to not contain any external references by this very function.

                            let td = cyclic_type_definitions
                                .iter()
                                .find(|td| td.id == id)
                                .expect("we should have a type definition for this id");
                            (td.id.clone(), td.name.clone())
                        })
                        .collect::<Vec<_>>();

                    for td in cyclic_type_definitions {
                        failed_type_definitions.push((
                            td,
                            RegistrationError::CircularReference {
                                cycle: cycle.clone(),
                            },
                        ));
                    }
                }

                // All the remaining type definitions are the ones that lead to circular
                // references but weren't part of the cycle.
                for (_, td) in postponed_type_definitions {
                    failed_type_definitions.push((td, RegistrationError::BlockedReference));
                }

                break;
            } else {
                last_count = type_definitions.len();
            }
        }

        (registered_type_definitions, failed_type_definitions)
    }

    fn insert_type_definition_instance(
        &mut self,
        type_definition_instance: TypeDefinitionInstance<Id, FieldName>,
    ) -> Arc<TypeDefinitionInstance<Id, FieldName>> {
        let type_definition_instance = Arc::new(type_definition_instance);

        self.by_id.insert(
            type_definition_instance.id.clone(),
            Arc::clone(&type_definition_instance),
        );
        self.by_name.insert(
            type_definition_instance.name.clone(),
            type_definition_instance.clone(),
        );

        type_definition_instance
    }
}

fn detect_minimal_cycle<Id: Ord + Clone>(dependencies: &BTreeMap<Id, BTreeSet<Id>>) -> Vec<Id> {
    let mut in_current_path: BTreeSet<Id> = BTreeSet::new();
    let mut parent: BTreeMap<Id, Id> = BTreeMap::new();
    let mut visited: BTreeSet<Id> = BTreeSet::new();

    fn dfs<Id: Ord + Clone>(
        node: Id,
        dependencies: &BTreeMap<Id, BTreeSet<Id>>,
        in_current_path: &mut BTreeSet<Id>,
        parent: &mut BTreeMap<Id, Id>,
        visited: &mut BTreeSet<Id>,
    ) -> Option<(Id, Id)> {
        visited.insert(node.clone());
        in_current_path.insert(node.clone());

        if let Some(neighbors) = dependencies.get(&node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    parent.insert(neighbor.clone(), node.clone());

                    if let Some(cycle) = dfs(
                        neighbor.clone(),
                        dependencies,
                        in_current_path,
                        parent,
                        visited,
                    ) {
                        return Some(cycle);
                    }
                } else if in_current_path.contains(neighbor) {
                    return Some((neighbor.clone(), node));
                }
            }
        }

        in_current_path.remove(&node);
        None
    }

    for node in dependencies.keys() {
        if !visited.contains(node) {
            if let Some((cycle_start, cycle_end)) = dfs(
                node.clone(),
                dependencies,
                &mut in_current_path,
                &mut parent,
                &mut visited,
            ) {
                let mut cycle = Vec::new();
                cycle.push(cycle_start.clone());

                let mut current = cycle_end.clone();
                while current != cycle_start {
                    cycle.push(current.clone());
                    current = parent.get(&current).expect("parent not found").clone();
                }

                cycle.push(cycle_start); // Close the cycle.
                cycle.reverse(); // Reverse the cycle to get the correct order.

                return cycle;
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use crate::type_attributes::{ArrayTypeAttributes, EnumTypeAttributes};

    use super::{RegistrationError, detect_minimal_cycle};

    type Id = u32;
    type FieldName = &'static str;
    type TypeDefinitionRegistry = super::TypeDefinitionRegistry<Id, FieldName>;
    type TypeDefinition = crate::TypeDefinition<Id, FieldName>;
    type TypeAttributes = crate::TypeAttributes<Id, FieldName>;

    #[test]
    fn test_type_definitions_registration() {
        let mut registry = TypeDefinitionRegistry::default();

        let my_int = TypeDefinition {
            id: 1,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Int32(Default::default()),
        };
        let my_string = TypeDefinition {
            id: 2,
            name: "MyString",
            description: None,
            attributes: TypeAttributes::String(Default::default()),
        };
        let my_int_array = TypeDefinition {
            id: 3,
            name: "MyIntArray",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(my_int.id)),
        };
        let my_string_array = TypeDefinition {
            id: 4,
            name: "MyStringArray",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(my_string.id)),
        };
        let my_int_dictionary = TypeDefinition {
            id: 5,
            name: "MyIntDictionary",
            description: None,
            attributes: TypeAttributes::Dictionary(
                crate::type_attributes::DictionaryTypeAttributes::new(my_string.id, my_int.id),
            ),
        };
        let my_enum = TypeDefinition {
            id: 6,
            name: "MyEnum",
            description: None,
            attributes: TypeAttributes::Enum(
                EnumTypeAttributes::builder()
                    .with_value("alpha")
                    .with_value("beta")
                    .with_value("gamma")
                    .build()
                    .unwrap(),
            ),
        };

        // This one will be registered later.
        let my_enum_array = TypeDefinition {
            id: 7,
            name: "MyEnumArray",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(my_enum.id)),
        };

        // Register the type definitions.
        let (registered, errors) = registry.register([
            my_int,
            my_string,
            my_int_array,
            my_string_array,
            my_int_dictionary,
            my_enum,
        ]);

        assert_eq!(
            registered.iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![1, 2, 6, 3, 4, 5],
        );
        assert!(errors.is_empty());

        // Register the enum array type definition.
        let (registered, errors) = registry.register([my_enum_array]);

        assert_eq!(
            registered.iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![7]
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_type_definitions_registration_broken_reference() {
        let mut registry = TypeDefinitionRegistry::default();

        let my_int = TypeDefinition {
            id: 1,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Int32(Default::default()),
        };
        let my_string_array = TypeDefinition {
            id: 4,
            name: "MyStringArray",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(
                2, /* THIS DOES NOT EXIST */
            )),
        };

        // Register the type definitions.
        let (registered, failed) = registry.register([my_int, my_string_array]);

        assert_eq!(
            registered.into_iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            failed
                .into_iter()
                .map(|(td, err)| (td.id, td.name, err))
                .collect::<Vec<_>>(),
            vec![(
                4,
                "MyStringArray",
                RegistrationError::BrokenReference { referenced_id: 2 }
            )]
        );
    }

    #[test]
    fn test_type_definitions_registration_duplicate_id() {
        let mut registry = TypeDefinitionRegistry::default();

        let my_int = TypeDefinition {
            id: 1,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Int32(Default::default()),
        };
        let my_string_array = TypeDefinition {
            id: 1,
            name: "MyStringArray",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(
                2, /* THIS DOES NOT EXIST */
            )),
        };

        // Register the type definitions.
        let (registered, failed) = registry.register([my_int, my_string_array]);

        assert_eq!(
            registered.into_iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            failed
                .into_iter()
                .map(|(td, err)| (td.id, td.name, err))
                .collect::<Vec<_>>(),
            vec![(
                1,
                "MyStringArray",
                RegistrationError::DuplicateTypeDefinition {
                    existing_name: "MyInt"
                }
            )]
        );
    }

    #[test]
    fn test_type_definitions_registration_duplicate_name() {
        let mut registry = TypeDefinitionRegistry::default();

        let my_int = TypeDefinition {
            id: 1,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Int32(Default::default()),
        };
        let my_string_array = TypeDefinition {
            id: 2,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(
                2, /* THIS DOES NOT EXIST */
            )),
        };

        // Register the type definitions.
        let (registered, failed) = registry.register([my_int, my_string_array]);

        assert_eq!(
            registered.into_iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            failed
                .into_iter()
                .map(|(td, err)| (td.id, td.name, err))
                .collect::<Vec<_>>(),
            vec![(
                2,
                "MyInt",
                RegistrationError::DuplicateTypeDefinitionName { existing_id: 1 }
            )]
        );
    }

    #[test]
    fn test_type_definitions_registration_circular_reference() {
        let mut registry = TypeDefinitionRegistry::default();

        let my_int = TypeDefinition {
            id: 1,
            name: "MyInt",
            description: None,
            attributes: TypeAttributes::Int32(Default::default()),
        };
        let my_array_a = TypeDefinition {
            id: 2,
            name: "MyArrayA",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(3)),
        };
        let my_array_b = TypeDefinition {
            id: 3,
            name: "MyArrayB",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(4)),
        };
        let my_array_c = TypeDefinition {
            id: 4,
            name: "MyArrayC",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(5)),
        };
        let my_array_d = TypeDefinition {
            id: 5,
            name: "MyArrayD",
            description: None,
            attributes: TypeAttributes::Array(ArrayTypeAttributes::new(3)),
        };

        // Register the type definitions.
        let (registered, failed) =
            registry.register([my_int, my_array_a, my_array_b, my_array_c, my_array_d]);

        assert_eq!(
            registered.into_iter().map(|td| td.id).collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(
            failed
                .into_iter()
                .map(|(td, err)| (td.id, td.name, err))
                .collect::<Vec<_>>(),
            vec![
                (
                    3,
                    "MyArrayB",
                    RegistrationError::CircularReference {
                        cycle: vec![
                            (3, "MyArrayB"),
                            (4, "MyArrayC"),
                            (5, "MyArrayD"),
                            (3, "MyArrayB")
                        ]
                    }
                ),
                (
                    4,
                    "MyArrayC",
                    RegistrationError::CircularReference {
                        cycle: vec![
                            (3, "MyArrayB"),
                            (4, "MyArrayC"),
                            (5, "MyArrayD"),
                            (3, "MyArrayB")
                        ]
                    }
                ),
                (
                    5,
                    "MyArrayD",
                    RegistrationError::CircularReference {
                        cycle: vec![
                            (3, "MyArrayB"),
                            (4, "MyArrayC"),
                            (5, "MyArrayD"),
                            (3, "MyArrayB")
                        ]
                    }
                ),
                (2, "MyArrayA", RegistrationError::BlockedReference),
            ]
        );
    }

    #[test]
    fn test_detect_minimal_cycle() {
        let deps = [(1, [2]), (2, [3]), (3, [1])]
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        let cycle = detect_minimal_cycle(&deps);
        assert_eq!(cycle, vec![1, 2, 3, 1]);

        let deps = [
            (1, vec![2, 3]),
            (2, vec![4, 5]),
            (3, vec![6, 7]),
            (4, vec![8]),
            (5, vec![9]),
            (6, vec![10]),
            (7, vec![11]),
            (8, vec![]),
            (9, vec![]),
            (10, vec![12]),
            (11, vec![]),
            (12, vec![3]),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();

        let cycle = detect_minimal_cycle(&deps);
        assert_eq!(cycle, vec![3, 6, 10, 12, 3]);

        let deps = [
            (1, vec![2, 3]),
            (2, vec![4, 5]),
            (3, vec![6, 7]),
            (4, vec![]),
            (5, vec![]),
            (6, vec![]),
            (7, vec![]),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();

        let cycle = detect_minimal_cycle(&deps);
        assert_eq!(cycle, Vec::<i32>::default());
    }
}
