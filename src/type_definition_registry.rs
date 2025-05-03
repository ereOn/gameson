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
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError<Id, FieldName> {
    /// A type definition with the same identifier already exists.
    #[error(
        "Unable to register type definition `{new_name}` with id `{id}` as another type definition `{existing_name}` with the same id already exists"
    )]
    DuplicateTypeDefinition {
        id: Id,
        new_name: FieldName,
        existing_name: FieldName,
    },

    /// A type definition with the same name already exists.
    #[error(
        "Unable to register type definition `{new_name}` with id `{id}` as another type definition with id `{existing_id}` has the same name"
    )]
    DuplicateTypeDefinitionName {
        id: Id,
        new_name: FieldName,
        existing_id: Id,
    },

    /// A type definition has a broken reference.
    #[error(
        "Unable to register type definition `{new_name}` with id `{id}` as it has a broken reference to type definition `{referenced_id}`"
    )]
    BrokenReference {
        id: Id,
        new_name: FieldName,
        referenced_id: Id,
    },

    /// A type definition has a circular reference.
    #[error(
        "Unable to register type definition `{new_name}` with id `{id}` as it would cause a circular reference cycle: {}",
        cycle.iter().map(|(id, name)| format!("`{name}` (`{id}`)")).join(" -> ")
    )]
    CircularReference {
        id: Id,
        new_name: FieldName,
        cycle: Vec<(Id, FieldName)>,
    },

    /// An error occurred while instantiating the type attributes.
    #[error(
        "unable to instantiate type attributes for type definition `{name}` with id `{id}`: {err}"
    )]
    InstantiationError {
        id: Id,
        name: FieldName,
        #[source]
        err: InstantiationError<Id, FieldName>,
    },
}

impl<Id: Ord + Clone + Display, FieldName: Ord + Clone + Display>
    TypeDefinitionRegistry<Id, FieldName>
{
    /// Register type definitions.
    ///
    /// The passed-in type definitions can only have references to other, previously registered,
    /// type definitions or to other types definitions in the same batch.
    ///
    /// If the batch contains broken or circular references, the function will return an error.
    ///
    /// If the batch contains duplicate type definitions, the function will return an error.
    pub fn register(
        &mut self,
        type_definitions: impl IntoIterator<Item = TypeDefinition<Id, FieldName>>,
    ) -> Result<(), RegistrationError<Id, FieldName>> {
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

        // While we have type definitions to register, we continue.
        while !type_definitions.is_empty() {
            // By sorting the definitions by the ascending number of references, we can ensure that the
            // first type definitions to be registered are the ones with the least number of
            // references and the lesser likelihood of broken or circular references.
            type_definitions.sort_by_key(|(refs, _)| refs.len());

            'outer: for (refs, td) in type_definitions {
                // Check for duplicate type definitions.
                if let Some(existing) = self.by_id.get(&td.id) {
                    return Err(RegistrationError::DuplicateTypeDefinition {
                        id: td.id.clone(),
                        new_name: td.name.clone(),
                        existing_name: existing.name.clone(),
                    });
                }

                if let Some(existing) = self.by_name.get(&td.name) {
                    return Err(RegistrationError::DuplicateTypeDefinitionName {
                        id: td.id.clone(),
                        new_name: td.name.clone(),
                        existing_id: existing.id.clone(),
                    });
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
                    Err(err) => {
                        return Err(RegistrationError::InstantiationError {
                            id: td.id,
                            name: td.name,
                            err,
                        });
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
                self.insert_type_definition_instance(type_definition_instance);
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
                for (refs, td) in &type_definitions {
                    for ref_ in refs {
                        if !(remaining_ids.contains(ref_) || self.by_id.contains_key(ref_)) {
                            return Err(RegistrationError::BrokenReference {
                                id: td.id.clone(),
                                new_name: td.name.clone(),
                                referenced_id: ref_.clone(),
                            });
                        }
                    }
                }

                // If we have no broken references, we have a circular reference.

                let deps = type_definitions
                    .iter()
                    .map(|(refs, td)| (td.id.clone(), refs.iter().cloned().collect()))
                    .collect::<BTreeMap<_, _>>();

                let cycle = detect_minimal_cycle(&deps)
                    .into_iter()
                    .map(|id| {
                        // It's impossible for the cycle to contain an id that is not in the new
                        // type definitions, as the already registered type definitions are
                        // guaranteed to not contain any external references by this very function.

                        let td = type_definitions
                            .iter()
                            .map(|(_, td)| td)
                            .find(|td| td.id == id)
                            .expect("we should have a type definition for this id");
                        (td.id.clone(), td.name.clone())
                    })
                    .collect::<Vec<_>>();

                let (id, new_name) = cycle
                    .first()
                    .expect("we should have a non-empty cycle")
                    .clone();

                return Err(RegistrationError::CircularReference {
                    id,
                    new_name,
                    cycle,
                });
            }

            last_count = type_definitions.len();
        }

        Ok(())
    }

    fn insert_type_definition_instance(
        &mut self,
        type_definition_instance: TypeDefinitionInstance<Id, FieldName>,
    ) {
        let type_definition_instance = Arc::new(type_definition_instance);

        self.by_id.insert(
            type_definition_instance.id.clone(),
            Arc::clone(&type_definition_instance),
        );
        self.by_name.insert(
            type_definition_instance.name.clone(),
            type_definition_instance,
        );
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
        registry
            .register([
                my_int,
                my_string,
                my_int_array,
                my_string_array,
                my_int_dictionary,
                my_enum,
            ])
            .expect("Failed to register type definitions");

        // Register the enum array type definition.
        registry
            .register([my_enum_array])
            .expect("Failed to register type definitions");
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
        match registry
            .register([my_int, my_string_array])
            .expect_err("should have failed")
        {
            RegistrationError::BrokenReference {
                id,
                new_name,
                referenced_id,
            } => {
                assert_eq!(id, 4);
                assert_eq!(new_name, "MyStringArray");
                assert_eq!(referenced_id, 2);
            }
            _ => panic!("should have been a broken reference error"),
        }
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
        match registry
            .register([my_int, my_string_array])
            .expect_err("should have failed")
        {
            RegistrationError::DuplicateTypeDefinition {
                id,
                new_name,
                existing_name,
            } => {
                assert_eq!(id, 1);
                assert_eq!(new_name, "MyStringArray");
                assert_eq!(existing_name, "MyInt");
            }
            _ => panic!("should have been a duplicate type definition error"),
        }
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
        match registry
            .register([my_int, my_string_array])
            .expect_err("should have failed")
        {
            RegistrationError::DuplicateTypeDefinitionName {
                id,
                new_name,
                existing_id,
            } => {
                assert_eq!(id, 2);
                assert_eq!(new_name, "MyInt");
                assert_eq!(existing_id, 1);
            }
            _ => panic!("should have been a duplicate type definition name error"),
        }
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
        match registry
            .register([my_int, my_array_a, my_array_b, my_array_c, my_array_d])
            .expect_err("should have failed")
        {
            RegistrationError::CircularReference {
                id,
                new_name,
                cycle,
            } => {
                assert_eq!(id, 3);
                assert_eq!(new_name, "MyArrayB");
                assert_eq!(
                    cycle,
                    vec![
                        (3, "MyArrayB"),
                        (4, "MyArrayC"),
                        (5, "MyArrayD"),
                        (3, "MyArrayB")
                    ]
                );
            }
            _ => panic!("should have been a circular reference error"),
        }
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
