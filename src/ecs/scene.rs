use std::{collections::HashMap, fmt::Debug};

use super::component::{Component, ComponentId};

pub struct Scene {
    components: HashMap<ComponentId, Component>,
}

impl Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("components", &self.components.len())
            .finish()
    }
}
