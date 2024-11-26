use wgpu::{Device, Queue};
use winit_input_helper::WinitInputHelper;

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
}

pub trait ComponentDetails {
    fn is_initialized(&self);
}
