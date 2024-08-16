use std::sync::{Arc, Mutex};

use sdl2::video::Window;
use smaa::SmaaTarget;

use crate::texture::Texture;

pub struct Renderer {
    pub surface: Arc<wgpu::Surface>,
    pub surface_format: wgpu::TextureFormat,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    pub depth_texture: Arc<Texture>,
    pub clear_color: wgpu::Color,
    pub smaa_target: Arc<Mutex<SmaaTarget>>,
}

impl Renderer {
    pub async fn new(
        window: &Window,
        clear_color: wgpu::Color,
        antialiasing: bool,
        limits: wgpu::Limits,
    ) -> Renderer {
        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = Arc::new(unsafe { instance.create_surface(window) }.unwrap());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS
                        | wgpu::Features::TEXTURE_BINDING_ARRAY
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits,
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            // present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = Arc::new(Texture::create_depth_texture(
            &device,
            &config,
            "depth_texture",
        ));

        let smaa_target = Arc::new(Mutex::new(SmaaTarget::new(
            &device,
            &queue,
            size.0,
            size.1,
            config.format,
            if antialiasing {
                smaa::SmaaMode::Smaa1X
            } else {
                smaa::SmaaMode::Disabled
            },
        )));

        Renderer {
            surface,
            surface_format,
            device,
            queue,
            config,
            size,
            depth_texture,
            clear_color,
            smaa_target,
        }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Arc::new(Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            ));
            self.smaa_target
                .clone()
                .lock()
                .unwrap()
                .resize(&self.device, new_size.0, new_size.1);
        }
    }
}
