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
        let imgui_context = Arc::new(Mutex::new(Context::create()));

        let mut imgui_context_lock = imgui_context.lock().unwrap();
        imgui_context_lock.set_ini_filename(None);
        imgui_context_lock.set_log_filename(None);
        imgui_context_lock
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let imgui_renderer = Arc::new(Mutex::new(Renderer::new(
            &mut imgui_context_lock,
            &device,
            &queue,
            config,
        )));

        let imgui_platform = Arc::new(Mutex::new(ImguiSdl2::new(&mut imgui_context_lock, window)));

        drop(imgui_context_lock);
        UiManager {
            imgui_context,
            imgui_renderer,
            imgui_platform,
        }
    }
}
