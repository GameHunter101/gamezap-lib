use winit::{
    application::ApplicationHandler,
    window::{Window, WindowAttributes},
};

#[derive(Debug, Default)]
/// Seperation of the windowing and basic event management
pub struct WindowAndEventManager {
    window_attributes: WindowAttributes,
    window: Option<Window>,
}

impl WindowAndEventManager {
    pub fn from_window_attributes(window_attributes: WindowAttributes) -> Self {
        Self {
            window_attributes,
            window: None,
        }
    }
}

impl ApplicationHandler for WindowAndEventManager {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(self.window_attributes.clone())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
