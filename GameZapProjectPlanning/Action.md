## **What Is It?**
- It describes a single "action" that the scene should execute
- An action must describe something that the [[Scene]] cannot do by itself
	- e.g. enabling/disabling a component
		- The [[Scene]] does not know which component it needs to toggle
## **How Does It Work?**
- Trait
	- Allows flexibility with future actions
	- With a single `Enum`, it would be very tedious to expand the available actions
	- Single method
		- Executes the action on a mutable reference of the current [[Scene]]
		- The action's data is stored in the implementation
- Each action must only perform a single "task"
	- e.g. the component toggler must never mess with UI or anything like that
	- Separation of concerns
```Rust
pub trait Action {
	fn execute(&self, scene: &mut Scene);
}
```
## **What Actions Are There?**
- Toggling [[Entity|entities]]
	- Data:
		- Entity ID
		- Should the entity be enabled
			- `Option<bool>`
			- `None` means toggle entity 
			- `true` means enable entity
			- `false` means disable entity
- Toggling [[Component|components]]
	- Data:
		- Component ID
		- Should the entity be enabled
			- `Option<bool>`
			- `None` means toggle component 
			- `true` means enable component
			- `false` means disable component
- Register UI data
	- Data:
		- Component ID
		- Text Buffer
- Set a [[Component|component's]] active material
	- Data:
		- Component ID
		- Global active material index
- Toggling [[Compute|compute shaders]]
	- Data:
		- Compute shader ID
		- Should the entity be enabled
			- `Option<bool>`
			- `None` means toggle component 
			- `true` means enable component
			- `false` means disable component
- Update [[Compute|compute shaders']] data
	- Data:
		- Compute shader ID
		- New compute data
			- Vector of tuples
				1) `Enum` with two variants
					1) Array data
					2) Texture data
						- Should probably specify which texture format it is
				2) Binding index of the target data being replaced
- Creating an [[Entity]]	
	- Data:
		- Entity parent ID
		- Vector of any components
		- Active material index
			- `Option<bool>`
- Creating a [[Component]]
	- Data:
		- Entity parent ID
		- The component itself
- Setting the active [[Camera]]
	- Data:
		- The camera component's ID
- Execute workload
	- Data:
		- Some async block
	- Note:
		- Some asynchronous block to be executed in the background through multiple iterations of the main loop
		- The component should constantly poll to see if the workload has finished executing. If it has, it should receive back the result of the workload (i.e. the poll function should return an `Option` where the `Some` value is the returned value)