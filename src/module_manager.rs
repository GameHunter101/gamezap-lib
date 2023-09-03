use std::cell::RefCell;

use nalgebra as na;

use crate::{
    camera::CameraManager, materials::MaterialManager, model::MeshManager,
    pipeline::PipelineManager,
};

pub struct ModuleManager {
    pub pipeline_manager: RefCell<PipelineManager>,
    pub material_manager: RefCell<MaterialManager>,
    pub mesh_manager: Option<RefCell<MeshManager>>,
    pub camera_manager: Option<RefCell<CameraManager>>,
}

impl ModuleManager {
    pub fn builder() -> ModuleManagerBuilder {
        ModuleManagerBuilder::default()
    }
    pub fn minimal() -> Self {
        ModuleManager {
            pipeline_manager: RefCell::new(PipelineManager::init()),
            material_manager: RefCell::new(MaterialManager::init()),
            mesh_manager: None,
            camera_manager: None,
        }
    }

    pub fn try_build_camera_resources(&self, device: &wgpu::Device) {
        if let Some(camera_manager) = &self.camera_manager {
            let mut camera_manager = camera_manager.borrow_mut();
            camera_manager.build_camera_resources(device);
        }
    }
}

pub struct ModuleManagerBuilder {
    pub mesh_manager: Option<RefCell<MeshManager>>,
    pub camera_manager: Option<RefCell<CameraManager>>,
}

impl ModuleManagerBuilder {
    pub fn mesh_manager(mut self) -> Self {
        let mesh_manager = RefCell::new(MeshManager::init());
        self.mesh_manager = Some(mesh_manager);
        self
    }

    pub fn camera_manager(
        mut self,
        camera_position: na::Vector3<f32>,
        pitch: f32,
        yaw: f32,
        fovy: f32,
        sensitivity: f32,
    ) -> Self {
        let camera_manager = RefCell::new(CameraManager::new(
            camera_position,
            pitch,
            yaw,
            fovy,
            sensitivity,
        ));
        self.camera_manager = Some(camera_manager);
        self
    }

    pub fn build(self) -> ModuleManager {
        ModuleManager {
            pipeline_manager: RefCell::new(PipelineManager::init()),
            material_manager: RefCell::new(MaterialManager::init()),
            mesh_manager: self.mesh_manager,
            camera_manager: self.camera_manager,
        }
    }
}

impl std::default::Default for ModuleManagerBuilder {
    fn default() -> Self {
        ModuleManagerBuilder {
            mesh_manager: None,
            camera_manager: None,
        }
    }
}
