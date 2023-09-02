use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use sdl2::video::Window;
use smaa::SmaaTarget;

use crate::{
    camera::{Camera, CameraUniform},
    materials::MaterialManager,
    model::{Mesh, MeshManager},
    pipeline::{Pipeline, PipelineManager, PipelineType},
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
    pub camera: Arc<Option<Mutex<Camera>>>,
    pub camera_uniform: Option<CameraUniform>,
    pub pipeline_manager: PipelineManager,
    pub mesh_manager: MeshManager,
    pub material_manager: Arc<Mutex<MaterialManager>>,
    pub smaa_target: Arc<Mutex<SmaaTarget>>,
}

impl Renderer {
    pub async fn new(
        window: Rc<Window>,
        clear_color: wgpu::Color,
        antialiasing: bool,
        material_manager: Option<Arc<Mutex<MaterialManager>>>,
    ) -> Renderer {
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
            device,
            queue,
            config,
            size,
            depth_texture,
            clear_color,
            camera: Arc::new(None),
            camera_uniform: None,
            pipeline_manager: PipelineManager::init(),
            mesh_manager: MeshManager::init(),
            material_manager: if let Some(material_manager) = material_manager {
                material_manager
            } else {
                Arc::new(Mutex::new(MaterialManager::init()))
            },
            smaa_target,
        }
    }

    pub fn set_camera(
        &mut self,
        camera: Arc<Option<Mutex<Camera>>>,
        camera_uniform: CameraUniform,
    ) {
        self.camera = camera;
        self.camera_uniform = Some(camera_uniform);
    }

    pub fn set_pipeline_manager(&mut self, pipeline_manager: PipelineManager) {
        self.pipeline_manager = pipeline_manager;
    }

    pub fn update_buffers(&mut self) {
        if let Some(camera_uniform) = &mut self.camera_uniform {
            let camera_clone = self.camera.as_ref().clone();
            if let Some(camera) = camera_clone {
                let camera = camera.lock().unwrap();
                camera_uniform.update_view_proj(&camera);
                self.queue.write_buffer(
                    &camera.buffer,
                    0,
                    bytemuck::cast_slice(&[*camera_uniform]),
                );
            }
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
            self.smaa_target
                .clone()
                .lock()
                .unwrap()
                .resize(&self.device, new_size.0, new_size.1);
        }
    }

    /// Initializes things the pipeline needs, such as pipelines
    pub fn prep_renderer(&mut self) {
        let camera_clone = self.camera.clone();
        let camera = match &*camera_clone {
            Some(camera) => Some(camera.lock().unwrap()),
            None => None,
        };
        self.pipeline_manager.create_pipelines(
            &self.device,
            self.config.format,
            camera,
            &self.material_manager.lock().unwrap(),
        );
    }

    fn bind_pipeline_and_resources<'a: 'b, 'b: 'c, 'c>(
        &'a self,
        pipeline_type: PipelineType,
        render_pass: &mut wgpu::RenderPass<'c>,
        camera_bind_group: Option<&'b wgpu::BindGroup>,
    ) {
        let material_manager = &self.material_manager.lock().unwrap();
        let pipeline = match pipeline_type {
            PipelineType::Plain => &self.pipeline_manager.plain_pipeline,
            PipelineType::DiffuseTexture => &self.pipeline_manager.diffuse_texture_pipeline,
            PipelineType::NormalDiffuseTexture => {
                &self.pipeline_manager.diffuse_normal_texture_pipeline
            }
        };
        if let Some(pipeline) = pipeline {
            render_pass.set_pipeline(&pipeline.pipeline);
            let mesh_array = match pipeline_type {
                PipelineType::Plain => &self.mesh_manager.plain_pipeline_models,
                PipelineType::DiffuseTexture => &self.mesh_manager.diffuse_pipeline_models,
                PipelineType::NormalDiffuseTexture => {
                    &self.mesh_manager.diffuse_normal_pipeline_models
                }
            };
            let material_array = match pipeline_type {
                PipelineType::Plain => &material_manager.plain_materials,
                PipelineType::DiffuseTexture => &material_manager.diffuse_texture_materials,
                PipelineType::NormalDiffuseTexture => {
                    &material_manager.diffuse_normal_texture_materials
                }
            };
            for material in material_array {
                let meshes_with_material: Vec<&Mesh> = mesh_array
                    .iter()
                    .filter(|mesh| mesh.material_index == material.material_index)
                    .collect();
                if let Some(camera_bind_group) = &camera_bind_group {
                    render_pass.set_bind_group(1, &camera_bind_group, &[]);
                }
                for mesh in meshes_with_material {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, mesh.transform_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                }
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let binding = self.smaa_target.clone();
        let mut smaa_clone = binding.lock().unwrap();
        let smaa_frame = smaa_clone.start_frame(&self.device, &self.queue, &view);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let camera_clone = &*self.camera.clone();
        let camera = if let Some(camera) = camera_clone {
            Some(camera.lock().unwrap())
        } else {
            None
        };
        let camera_bind_group = if let Some(camera) = &camera {
            Some(&camera.bind_group)
        } else {
            None
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &(*smaa_frame),
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

            self.bind_pipeline_and_resources(
                PipelineType::Plain,
                &mut render_pass,
                camera_bind_group,
            );
            self.bind_pipeline_and_resources(
                PipelineType::DiffuseTexture,
                &mut render_pass,
                camera_bind_group,
            );

            /*             if let Some(no_texture_pipeline) = &self.pipeline_manager.plain_pipeline {
                render_pass.set_pipeline(&no_texture_pipeline.pipeline);
                for material in &self.material_manager.plain_materials {
                    render_pass.set_bind_group(0, &material.bind_group, &[]);
                    if let Some(camera_bind_group) = &camera_bind_group {
                        render_pass.set_bind_group(1, &camera_bind_group, &[]);
                    }
                    for mesh in &material.meshes {
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

            if let Some(diffuse_texture_pipeline) = &self.pipeline_manager.diffuse_texture_pipeline
            {
                render_pass.set_pipeline(&diffuse_texture_pipeline.pipeline);
                for material in &self.material_manager.diffuse_texture_materials {
                    if let Some(camera_bind_group) = &camera_bind_group {
                        render_pass.set_bind_group(1, &camera_bind_group, &[]);
                    }
                    for mesh in &material.meshes {
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
            } */
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        smaa_frame.resolve();
        output.present();

        Ok(())
    }
}
