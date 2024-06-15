use std::{
    any::{Any, TypeId},
    rc::Rc,
    sync::{Arc, Mutex},
};

use gamezap::{
    ecs::{
        component::{ComponentId, ComponentSystem},
        concepts::ConceptManager,
        entity::{Entity, EntityId},
        material::Material,
        scene::AllComponents,
    },
    EngineDetails, EngineSystems,
};

use wgpu::{Device, Queue};

#[derive(Debug, Clone)]
pub struct TransparencyComponent {
    parent: EntityId,
    id: ComponentId,
}

impl Default for TransparencyComponent {
    fn default() -> Self {
        TransparencyComponent {
            parent: EntityId::MAX,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for TransparencyComponent {
    fn update(
        &mut self,
        _device: Arc<Device>,
        queue: Arc<Queue>,
        _component_map: &mut AllComponents,
        engine_details: Rc<Mutex<EngineDetails>>,
        _engine_systems: Rc<Mutex<EngineSystems>>,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _active_camera_id: Option<EntityId>,
        _entities: &mut Vec<Entity>,
        materials: Option<&(Vec<Material>, usize)>,
    ) {
        let details = engine_details.lock().unwrap();
        let time = details.time_elapsed.as_secs_f32();

        let materials = materials.unwrap();
        let selected_material = &materials.0[materials.1];
        if let Some((_, buffer)) = &selected_material.uniform_buffer_bind_group() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[time % 2.0]));
        }
    }

    fn get_parent_entity(&self) -> EntityId {
        self.parent
    }

    fn get_id(&self) -> ComponentId {
        self.id
    }

    fn update_metadata(&mut self, parent: EntityId, same_component_count: u32) {
        self.parent = parent;
        self.id.0 = parent;
        self.id.2 = same_component_count;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
