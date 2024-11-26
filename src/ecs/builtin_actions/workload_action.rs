use std::{any::Any, fmt::Debug, future::Future, pin::Pin};

use crate::ecs::{actions::Action, component::ComponentId, scene::Scene};

pub type Workload = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;

pub struct WorkloadAction(ComponentId, Workload);

impl Debug for WorkloadAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WorkloadAction").field(&self.0).field(&"Future").finish()
    }
}

impl<'a> Action<'a> for WorkloadAction {
    fn execute(&'a mut self, scene: &'a mut Scene<'a>) {
        scene.attach_workload(self.0, &mut self.1);
    }
}
