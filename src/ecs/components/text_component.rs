use crate::{ecs::scene::TextParams, new_component};

new_component!(TextComponent {
    params: TextParams,
    text_index: usize
});

impl TextComponent {
    pub fn new(params: TextParams) -> Self {
        Self {
            params,
            text_index: usize::MAX,
            parent: 0,
            id: (EntityId::MAX, TypeId::of::<Self>(), 0),
        }
    }
}

impl ComponentSystem for TextComponent {
    fn initialize(
        &mut self,
        _device: Arc<Device>,
        _queue: Arc<Queue>,
        _component_map: &AllComponents,
        _concept_manager: Rc<Mutex<ConceptManager>>,
        _engine_details: Option<Rc<Mutex<EngineDetails>>>,
        _engine_systems: Option<Rc<Mutex<EngineSystems>>>,
        _ui_manager: Rc<Mutex<crate::ui_manager::UiManager>>,
        text_items: &mut Vec<TextParams>,
    ) {
        self.text_index = text_items.len();
        text_items.push(self.params.clone());
    }
}
