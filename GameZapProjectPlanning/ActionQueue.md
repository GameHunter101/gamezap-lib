## **What Is It?**
- Stores a collection of [[Action]]s
	- [[Action]]s describe communications that components make with the scene itself
## **How Does It Do It?**
```Rust
#[derive(Debug)]
pub trait Action {
	fn execute(&self, &mut Scene)
}
```

```Rust
#[derive(Debug)]
pub struct ActionQueue {
	actions: Vec<Box<dyn Action>>
}

impl ActionQueue {
	pub fn execute_events(&self, current_scene: &mut Scene) -> {
		for event in self.actions {
			event.execute(current_scene);
		}
	}
}

impl FromIterator<Event> for ActionQueue {
	...
}
```