use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use log::error;

#[derive(Debug)]
/// Seperation of the windowing and basic event management
pub struct WindowAndEventManager {
    pub glfw_context: Glfw,
    pub window: PWindow,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
}

impl Default for WindowAndEventManager {
    fn default() -> Self {
        Self::from_window_attributes(800, 600, "GameZap Project", glfw::WindowMode::Windowed)
    }
}

impl WindowAndEventManager {
    pub fn from_window_attributes(
        width: u32,
        height: u32,
        title: &str,
        mode: glfw::WindowMode,
    ) -> Self {
        let mut glfw_context =
            glfw::init(Self::glfw_error_callback).expect("Failed to initialize GLFW context");
        let (mut window, events) = glfw_context
            .create_window(width, height, title, mode)
            .expect("Failed to create GLFW window");

        window.make_current();
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);

        Self {
            glfw_context,
            window,
            events,
        }
    }

    fn glfw_error_callback(err: glfw::Error, description: String) {
        error!("GLFW error {:?}: {:?}", err, description);
    }

}
