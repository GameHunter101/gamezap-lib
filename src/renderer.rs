use std::sync::{Arc, Mutex};

use sdl2::video::Window;
use smaa::SmaaTarget;

use crate::texture::Texture;

pub struct Renderer {
    pub surface: Arc<wgpu::Surface>,
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
        window: Arc<Window>,
        clear_color: wgpu::Color,
        antialiasing: bool,
    ) -> Renderer {
        let size = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = Arc::new(unsafe { instance.create_surface(&*window) }.unwrap());

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
                        | wgpu::Features::TEXTURE_BINDING_ARRAY,
                    limits: wgpu::Limits::default(),
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
            present_mode: surface_caps.present_modes[0],
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

        // module_manager.try_build_camera_resources(&device);

        Renderer {
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            clear_color,
            smaa_target,
            // module_manager,
        }
    }

    /* pub fn update_buffers(&self) {
        if let Some(camera_manager) = &self.module_manager.camera_manager {
            let camera_manager = camera_manager.borrow();
            camera_manager
                .camera_uniform
                .borrow_mut()
                .update_view_proj(camera_manager.camera.borrow());
            self.queue.write_buffer(
                &camera_manager.buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&[camera_manager.camera_uniform.borrow().to_owned()]),
            );
        }
    } */

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

    /* /// Initializes things the pipeline needs, such as pipelines
    pub fn prep_renderer(&self) {
        let camera_manager = if let Some(camera_manager) = &self.module_manager.camera_manager {
            Some(camera_manager.borrow())
        } else {
            None
        };

        self.module_manager
            .pipeline_manager
            .borrow_mut()
            .create_pipelines(
                self.device.clone(),
                self.config.format,
                self.module_manager.material_manager.borrow(),
                camera_manager,
            );
    } */

    /* fn bind_pipeline_and_resources<'a: 'b, 'b: 'c, 'c>(
        &'a self,
        pipeline_type: PipelineType,
        render_pass: &mut wgpu::RenderPass<'c>,
        pipeline_manager: &'b Ref<PipelineManager>,
        material_manager: &'b Ref<MaterialManager>,
        mesh_manager: &'b Option<MutexGuard<MeshManager>>,
        camera_manager: &'b Option<Ref<CameraManager>>,
    ) {
        // let pipeline_manager = self.module_manager.pipeline_manager.borrow();
        let pipeline = match pipeline_type {
            PipelineType::Plain => &pipeline_manager.plain_pipeline,
            PipelineType::DiffuseTexture => &pipeline_manager.diffuse_texture_pipeline,
            PipelineType::NormalDiffuseTexture => &pipeline_manager.diffuse_normal_texture_pipeline,
        };
        if let Some(pipeline) = pipeline {
            render_pass.set_pipeline(&pipeline.pipeline);
            if let Some(mesh_manager) = mesh_manager {
                // let material_manager = self.module_manager.material_manager.borrow();

                let mesh_array = match pipeline_type {
                    PipelineType::Plain => &mesh_manager.plain_pipeline_models,
                    PipelineType::DiffuseTexture => &mesh_manager.diffuse_pipeline_models,
                    PipelineType::NormalDiffuseTexture => {
                        &mesh_manager.diffuse_normal_pipeline_models
                    }
                };

                let material_array = material_manager.get_pipeline_materials(pipeline_type);
                for material in material_array {
                    let meshes_with_material: Vec<&Arc<Mesh>> = mesh_array
                        .iter()
                        .filter(|mesh| mesh.material_index == material.material_index)
                        .collect();

                    render_pass.set_bind_group(0, &material.bind_group, &[]);
                    if let Some(camera_manager) = camera_manager {
                        let camera_bind_group = camera_manager.bind_group.as_ref().unwrap();
                        render_pass.set_bind_group(1, &camera_bind_group, &[]);
                    }

                    for mesh in meshes_with_material {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, mesh.transform_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
                    }
                }
            }
        }
    } */

    /* pub async fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let binding = self.smaa_target.clone();
        let mut smaa_clone = binding.lock().unwrap();
        let smaa_frame = smaa_clone.start_frame(&self.device, &self.queue, &view);

        let mut render_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

        let pipeline_manager = self.module_manager.pipeline_manager.borrow();
        let material_manager = self.module_manager.material_manager.borrow();
        let mesh_manager = if let Some(mesh_manager) = &self.module_manager.mesh_manager {
            Some(mesh_manager.lock().unwrap())
        } else {
            None
        };
        let camera_manager = if let Some(camera_manager) = &self.module_manager.camera_manager {
            Some(camera_manager.borrow())
        } else {
            None
        };

        {
            let mut render_pass = render_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                &pipeline_manager,
                &material_manager,
                &mesh_manager,
                &camera_manager,
            );
            self.bind_pipeline_and_resources(
                PipelineType::DiffuseTexture,
                &mut render_pass,
                &pipeline_manager,
                &material_manager,
                &mesh_manager,
                &camera_manager,
            );
        }
        self.queue.submit(std::iter::once(render_encoder.finish()));

        let compute_manager = self.module_manager.compute_manager.borrow();

        compute_manager
            .run_passive_shaders(self.device.clone(), self.queue.clone())
            .await;

        smaa_frame.resolve();
        output.present();

        Ok(())
    } */
}
