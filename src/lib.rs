use engine_management::{
    rendering_management::RenderingManager, window_and_event_management::WindowAndEventManager,
};
use glfw::{Context, WindowEvent};

pub mod engine_management {
    pub mod rendering_management;
    pub mod window_and_event_management;
}

pub mod engine_support {
    pub mod texture_support;
}

// #[derive(Debug)]
/// The main engine struct. Contains the state for the whole engine.
pub struct Gamezap {
    window_and_event_manager: WindowAndEventManager,
    rendering_manager: RenderingManager,
}

impl Gamezap {
    pub fn builder() -> GamezapBuilder {
        GamezapBuilder::default()
    }

    pub async fn main_loop(mut self) {
        while !self.window_and_event_manager.window.should_close() {
            self.window_and_event_manager.glfw_context.poll_events();
            for (_, event) in glfw::flush_messages(&self.window_and_event_manager.events) {
                match event {
                    glfw::WindowEvent::MouseButton(
                        glfw::MouseButton::Button1,
                        glfw::Action::Press,
                        _,
                    ) => {
                        println!("pressed, {event:?}");
                    }
                    _ => {}
                }
            }
        }

        tokio::task::spawn(async move {
            self.rendering_manager.render();
            self.window_and_event_manager.window.swap_buffers();
        });

    }
}

// #[derive(Debug)]
pub struct GamezapBuilder {
    window_and_event_manager: WindowAndEventManager,
    antialiasing_enabled: bool,
    clear_color: wgpu::Color,
}

impl Default for GamezapBuilder {
    fn default() -> Self {
        Self {
            window_and_event_manager: WindowAndEventManager::default(),
            antialiasing_enabled: false,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

impl GamezapBuilder {
    pub fn window_settings(
        mut self,
        width: u32,
        height: u32,
        title: &str,
        mode: glfw::WindowMode,
    ) -> Self {
        self.window_and_event_manager =
            WindowAndEventManager::from_window_attributes(width, height, title, mode);
        self
    }

    pub fn antialiasing_enabled(mut self, enabled: bool) -> Self {
        self.antialiasing_enabled = enabled;
        self
    }

    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.clear_color = color;
        self
    }

    pub async fn build(self) -> Gamezap {
        let window_and_event_manager = self.window_and_event_manager;

        let rendering_manager = RenderingManager::new(&window_and_event_manager.window, self.antialiasing_enabled, self.clear_color).await;

        Gamezap {
            rendering_manager,
            window_and_event_manager,
        }
    }
}
