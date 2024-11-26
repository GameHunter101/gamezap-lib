use super::material::MaterialId;

pub type EntityId = u32;

#[derive(Debug)]
pub struct Entity {
    id: EntityId,
    children_ids: Vec<EntityId>,
    is_enabled: bool,
    materials: Vec<MaterialId>,
    active_material: MaterialId,
}
