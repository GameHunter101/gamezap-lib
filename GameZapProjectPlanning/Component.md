## **State**
- Persistent
- Local
- Should be able to store references
	- Maybe not mutable references, TBD
---
## **System**
- Should not have any limitations beyond [trait object limitations](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
	- YES ASYNC FUNCTIONS (make sure to use `#[async_trait::async_trait]`
#### Update
- Communicate with other components
- Access and mutate the current component's state and functions
- Access and mutate various properties of the scene
	- Active camera
	- Enable / Disable Components
	- Enable / Disable Entities
	- Mutation done through [[Action]]s submitted to an [[ActionQueue]]
	- Accessing done through 