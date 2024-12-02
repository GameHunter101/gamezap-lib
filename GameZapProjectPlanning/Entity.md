## **What Does It Do?**
- Stores various metadata about the game object
## **What Does It Store**
- `entity_id`
	- Unique integer representing the entity
	- Assigned by the [[Scene]]
- `children_ids`
	- Vector containing entity IDs of child entities
	- Updated at the end of the current [[update]] iteration
- `parent_entity_id`
	- The entity ID corresponding to the entity's parent
	- Updated at the end of the current [[update]] iteration
- `is_enabled`
	- A boolean that dictates whether or not the entity's [[Component]]s get updated or not
	- Updated at the end of the current [[update]] iteration
- `active_material`
	- The material ID corresponding to the material currently visible on the entity
	- Updated at the end of the current [[update]] iteration