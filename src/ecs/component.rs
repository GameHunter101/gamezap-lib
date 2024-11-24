use wgpu::{Device, Queue};
use winit_input_helper::WinitInputHelper;

pub type ComponentId = u32;

pub type Component = Box<dyn ComponentSystem>;

pub trait ComponentSystem {
    fn initialize(&self) {}
    fn update(&mut self, device: &Device, queue: &Queue, input_manager: &WinitInputHelper) {}
    fn render(&self, device: &Device, queue: &Queue) {}
}
