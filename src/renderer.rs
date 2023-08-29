use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use sdl2::video::Window;

use crate::{
    camera::{Camera, CameraUniform},
    pipeline_manager::PipelineManager,
    texture::Texture,
};

pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    pub depth_texture: Texture,
    pub clear_color: wgpu::Color,
    pub camera: Option<Arc<Mutex<Camera>>>,
    pub camera_uniform: Option<CameraUniform>,
    pub pipeline_manager: Option<Arc<Mutex<PipelineManager>>>,
}

impl Renderer {
    pub async fn new(window: Rc<Window>, clear_color: wgpu::Color) -> Renderer {
        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&*window) }.unwrap();

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
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        Renderer {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            clear_color,
            camera: None,
            camera_uniform: None,
            pipeline_manager: None,
        }
    }

    pub fn set_camera(&mut self, camera: Arc<Mutex<Camera>>, camera_uniform: CameraUniform) {
        self.camera = Some(camera);
        self.camera_uniform = Some(camera_uniform);
    }

    pub fn set_pipeline_manager(&mut self, pipeline_manager: Arc<Mutex<PipelineManager>>) {
        self.pipeline_manager = Some(pipeline_manager)
    }

    pub fn update_buffers(&mut self) {
        if let Some(camera_uniform) = &mut self.camera_uniform {
            let camera_clone = self.camera.as_ref().unwrap().clone();
            let camera = camera_clone.lock().unwrap();
            camera_uniform.update_view_proj(&camera);
            self.queue
                .write_buffer(&camera.buffer, 0, bytemuck::cast_slice(&[*camera_uniform]));
        }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn create_pipelines(&mut self) {
        if let Some(pipeline_manager) = &mut self.pipeline_manager {
            pipeline_manager.lock().unwrap().create_pipelines(
                &self.device,
                self.config.format,
                &self.camera.as_ref().unwrap().lock().unwrap(),
            )
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        // TODO:
        // Catch if camera is None
        let camera_clone = self.camera.as_ref().unwrap().clone();
        let camera = camera_clone.lock().unwrap();

        // TODO:
        // Catch if pipeline is None
        let pipeline_manager_clone = self.pipeline_manager.as_ref().unwrap().clone();
        let pipeline_manager = pipeline_manager_clone.lock().unwrap();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // if let Some(pipeline_manager) = &self.pipeline_manager {
            if let Some(no_texture_pipeline) = &pipeline_manager.no_texture_pipeline {
                render_pass.set_pipeline(&no_texture_pipeline.pipeline);
                for material in &pipeline_manager.materials.no_texture_materials {
                    render_pass.set_bind_group(0, &material.bind_group, &[]);
                    render_pass.set_bind_group(1, &camera.bind_group, &[]);
                    for (i, mesh) in material.meshes.iter().enumerate() {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, mesh.transform_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                    }
                }
            }

            if let Some(diffuse_texture_pipeline) = &pipeline_manager.diffuse_texture_pipeline {
                render_pass.set_pipeline(&diffuse_texture_pipeline.pipeline);
                for material in &pipeline_manager.materials.diffuse_texture_materials {
                    render_pass.set_bind_group(1, &camera.bind_group, &[]);
                    for (i, mesh) in material.meshes.iter().enumerate() {
                        render_pass.set_bind_group(0, &material.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, mesh.transform_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                    }
                }
            }

            // for (i, mesh) in materials.meshes.iter().enumerate() {
            //     render_pass.set_vertex_buffer(i as u32, mesh.vertex_buffer.slice(..));
            //     render_pass.set_index_buffer(
            //         mesh.index_buffer.slice(..),
            //         wgpu::IndexFormat::Uint16,
            //     );
            // }

            // render_pass.set_bind_group(0, &materials.material.bind_group, &[]);
            // render_pass.set_bind_group(1, &materials.camera_bind_group, &[]);
            // render_pass.draw_indexed(0..materials.num_indices, 0, 0..1);
            // }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
