use engine_management::window_and_event_management::WindowAndEventManager;
use winit::{event_loop::EventLoop, window::WindowAttributes};

pub mod engine_management {
    pub mod window_and_event_management;
}

#[derive(Debug)]
/// The main engine struct. Contains the state for the whole engine.
pub struct Gamezap {
    event_loop: EventLoop<()>,
    window_event_manager: WindowAndEventManager,
}

impl Default for Gamezap {
    fn default() -> Self {
        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        Self { event_loop, window_event_manager: WindowAndEventManager::default() }
    }
}

impl Gamezap{
    pub fn new(window_attributes: WindowAttributes) -> Self {
        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        Self { event_loop, window_event_manager: WindowAndEventManager::from_window_attributes(window_attributes) }
    }

    pub fn main_loop(mut self) {
        let manager= &mut self.window_event_manager;
        if let Err(err) = self.event_loop.run_app(manager) {
            panic!("Error executing event loop. {err}");
        }
    }
}

