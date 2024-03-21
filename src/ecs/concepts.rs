use std::{any::Any, collections::HashMap, fmt::Debug};

use super::{component::ComponentId, entity::EntityId};

#[derive(Debug)]
pub enum ConceptManagerError {
    ComponentNotFound(ComponentId),
    ConceptNotFound(String),
    DowncastFailed,
}

#[derive(Debug, Default)]
pub struct ConceptManager {
    pub concepts: HashMap<ComponentId, HashMap<String, Box<dyn Any>>>,
}

impl ConceptManager {
    pub fn register_concept<T: Any>(
        &mut self,
        component: ComponentId,
        name: String,
        data: Box<T>,
    ) -> String {
        match self.concepts.get_mut(&component) {
            Some(concepts_map) => {
                concepts_map.insert(name.clone(), data);
            }
            None => {
                let mut map: HashMap<String, Box<dyn Any>> = HashMap::new();
                map.insert(name.clone(), data);
                self.concepts.insert(component, map);
            }
        };
        name
    }

    pub fn get_concept<T: Any + Debug>(
        &self,
        component: ComponentId,
        concept_name: String,
    ) -> Result<&T, ConceptManagerError> {
        let component_concepts = &self.concepts.get(&component);
        match component_concepts {
            Some(concepts_map) => match concepts_map.get(&concept_name) {
                Some(concept) => {
                    let concept_ref_option = concept.downcast_ref::<T>();
                    match concept_ref_option {
                        Some(concept_ref) => Ok(concept_ref),
                        None => Err(ConceptManagerError::DowncastFailed)
                    }
                }
                None => Err(ConceptManagerError::ConceptNotFound(concept_name)),
            },
            None => Err(ConceptManagerError::ComponentNotFound(component)),
        }
    }

    pub fn get_concept_mut<T: Any + Debug>(
        &mut self,
        component: ComponentId,
        concept_name: String,
    ) -> Result<&mut T, ConceptManagerError> {
        let component_concepts = self.concepts.get_mut(&component);
        match component_concepts {
            Some(concepts_map) => match concepts_map.get_mut(&concept_name) {
                Some(concept) => {
                    let concept_mut_option = concept.downcast_mut::<T>();
                    match concept_mut_option {
                        Some(concept_mut) => Ok(concept_mut),
                        None => Err(ConceptManagerError::DowncastFailed)
                    }
                }
                None => Err(ConceptManagerError::ConceptNotFound(concept_name)),
            },
            None => Err(ConceptManagerError::ComponentNotFound(component)),
        }
    }

    pub fn register_component_concepts(
        &mut self,
        component: ComponentId,
        concepts: HashMap<String, Box<dyn Any>>,
    ) -> Vec<String> {
        let names = concepts.keys().cloned().collect();
        self.concepts.insert(component, concepts);
        names
    }

    pub fn modify_key(&mut self, old_id: ComponentId, new_id: ComponentId) {
        if let Some(concepts) = self.concepts.remove(&old_id) {
            self.concepts.insert(new_id, concepts);
        }
    }
}

/*
* Plans:
* - make a macro that creates a new struct & registers concepts from dev input
* - the update trait method is passed in a queue object that stores any entity additions/deletions
*   in corresponding hash maps
* - only completes these entity modifications after the end of the update loop
* */
