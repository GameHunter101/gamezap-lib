## **What Does It Do?**
- Stores various metadata about the game object
## **What Does It Store**
- `entity_id`
	- Unique integer representing the entity
	- Assigned by the [[Scene]]
- `children_ids`
	- Vector containing entity IDs of child entities
	- Updated at the end of the current [[update]] iteration
- `is_enabled`
	- A boolean that dictates whether or not the entity's [[Component]]s get updated or not
	- Updated at the end of the current [[update]] iteration
- `materials`
	- A Vector of all of indices representing the materials assigned to the entity
	- Updated at the end of the current [[update]] iteration
- `active_material`
	- An integer representing the index of the active material
	- REPRESENTS THE LOCAL MATERIAL INDEX, NOT A GLOBAL INDEX
	- Updated at the end of the current [[update]] iteration