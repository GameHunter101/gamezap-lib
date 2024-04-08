use std::sync::{Arc, Mutex};

use imgui::Context;
use imgui_sdl2::ImguiSdl2;
use imgui_wgpu::{Renderer, RendererConfig};
use sdl2::video::Window;
use wgpu::{Device, Queue, TextureFormat};

#[allow(unused)]
pub struct UiManager {
    pub imgui_context: Arc<Mutex<Context>>,
    pub imgui_renderer: Arc<Mutex<Renderer>>,
    pub imgui_platform: Arc<Mutex<ImguiSdl2>>,
}

impl UiManager {
    pub fn new(
        texture_format: TextureFormat,
        device: Arc<Device>,
        queue: Arc<Queue>,
        window: &Window,
    ) -> Self {
        let config = RendererConfig {
            texture_format,
            ..Default::default()
        };
        let mut imgui_context = Context::create();

        imgui_context.set_ini_filename(None);
        imgui_context.set_log_filename(None);
        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let imgui_renderer = Renderer::new(&mut imgui_context, &device, &queue, config);

        let imgui_platform = ImguiSdl2::new(&mut imgui_context, window);

        UiManager {
            imgui_context: Arc::new(Mutex::new(imgui_context)),
            imgui_renderer: Arc::new(Mutex::new(imgui_renderer)),
            imgui_platform: Arc::new(Mutex::new(imgui_platform)),
        }
    }
}

impl std::fmt::Debug for UiManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiManager")
            .field("imgui_context", &self.imgui_context)
            .field(
                "imgui_renderer",
                &String::from("Renderer") as &dyn std::fmt::Debug,
            )
            .field(
                "imgui_platform",
                &String::from("Platform") as &dyn std::fmt::Debug,
            )
            .finish()
    }
}
