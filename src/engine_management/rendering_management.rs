use std::fmt::Debug;

use smaa::SmaaTarget;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::engine_support::texture_support;

pub struct RenderingManager {
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    width: u32,
    height: u32,
    depth_texture: texture_support::Texture,
    clear_color: wgpu::Color,
    smaa_target: SmaaTarget,
}

impl<'a> RenderingManager {
    pub async fn new(
        window: &'a glfw::Window,
        antialiasing_enabled: bool,
        clear_color: wgpu::Color,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let window_size = window.get_size();

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: window.display_handle().unwrap().into(),
                raw_window_handle: window.window_handle().unwrap().into(),
            })
        }
        .expect("Error creating the surface for the given window.");

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
                    label: Some("Renderer device descriptor"),
                    memory_hints: wgpu::MemoryHints::Performance,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window_size.0 as u32,
            height: window_size.1 as u32,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let depth_texture = texture_support::Texture::create_depth_texture(&device, &config);

        let smaa_target = SmaaTarget::new(
            &device,
            &queue,
            window_size.0 as u32,
            window_size.1 as u32,
            config.format,
            if antialiasing_enabled {
                smaa::SmaaMode::Smaa1X
            } else {
                smaa::SmaaMode::Disabled
            },
        );

        Self {
            surface,
            format,
            device,
            queue,
            config,
            width: window_size.0 as u32,
            height: window_size.1 as u32,
            depth_texture,
            clear_color,
            smaa_target,
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let smaa_frame = self
            .smaa_target
            .start_frame(&self.device, &self.queue, &view);

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &smaa_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view_ref(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        }
        smaa_frame.resolve();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = texture_support::Texture::create_depth_texture(&self.device, &self.config);
        self.smaa_target.resize(&self.device, width, height);
    }
}

impl Debug for RenderingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderingManager")
            .field("surface", &self.surface)
            .field("format", &self.format)
            .field("device", &self.device)
            .field("queue", &self.queue)
            .field("config", &self.config)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("depth_texture", &self.depth_texture)
            .field("clear_color", &self.clear_color)
            .field("smaa_target", &"smaa target".to_string())
            .finish()
    }
}
