use wgpu::{Device, Queue};
use winit_input_helper::WinitInputHelper;

use super::entity::EntityId;

pub type ComponentId = u32;

pub type Component = Box<dyn ComponentSystem + Send>;

#[async_trait::async_trait]
pub trait ComponentSystem: ComponentDetails {
    fn initialize(&self) {}
    async fn update(
        &mut self,
        device: &Device,
        queue: &Queue,
        input_manager: &WinitInputHelper,
        other_components: &[&mut Component],
    ) {
    }
    fn render(&self, device: &Device, queue: &Queue) {}

    /// Lower order means it is rendered earlier
    fn rendering_order(&self) -> i32 {0}
}

pub trait ComponentDetails {
    fn is_initialized(&self) -> bool;
    
    fn parent_entity(&self) -> EntityId;

    fn set_parent_entity(&mut self, parent_id: EntityId);
}
