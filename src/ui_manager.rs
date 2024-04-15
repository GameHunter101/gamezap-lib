use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::Path,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use imgui::{Context, FontId};
use imgui_sdl2::ImguiSdl2;
use imgui_wgpu::{Renderer, RendererConfig};
use sdl2::video::Window;
use wgpu::{Device, Queue, TextureFormat};

#[derive(Debug)]
pub enum UiError {
    FontFileLoadingError,
}

#[allow(unused)]
pub struct UiManager {
    pub imgui_context: Rc<Mutex<Context>>,
    pub imgui_renderer: Rc<Mutex<Renderer>>,
    pub imgui_platform: Rc<Mutex<ImguiSdl2>>,
    pub render_flag: Rc<AtomicBool>,

    pub font_ids: HashMap<String, FontId>,
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
            imgui_context: Rc::new(Mutex::new(imgui_context)),
            imgui_renderer: Rc::new(Mutex::new(imgui_renderer)),
            imgui_platform: Rc::new(Mutex::new(imgui_platform)),
            render_flag: Rc::new(AtomicBool::new(false)),
            font_ids: HashMap::new(),
        }
    }

    pub fn set_render_flag(&mut self) {
        self.render_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn clear_render_flag(&mut self) {
        self.render_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn load_font(
        &mut self,
        font_name: &str,
        path: String,
        size_pixels: f32,
    ) -> Result<FontId, UiError> {
        let mut imgui_context = self.imgui_context.lock().unwrap();
        let bytes = Self::read_ttf_bytes(path)?;
        let font_id = imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::TtfData {
                data: &bytes,
                size_pixels,
                config: None,
            }]);
        self.font_ids.insert(font_name.to_string(), font_id);
        Ok(font_id)
    }

    pub fn read_ttf_bytes(path: String) -> Result<Vec<u8>, UiError> {
        let path = Path::new(&std::env::current_dir().unwrap())
            .join(path);
        match File::open(&path) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                match file.read_to_end(&mut buffer) {
                    Ok(_) => return Ok(buffer),
                    Err(err) => {
                        dbg!(err);
                    }
                }
            }
            Err(err) => {
                dbg!(path);
                dbg!(err);
            }
        };
        Err(UiError::FontFileLoadingError)
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
