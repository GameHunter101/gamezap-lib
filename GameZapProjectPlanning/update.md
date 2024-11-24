## **What Does It Do?**
Executes the [[Component#Update]] function for all enabled components in all enabled entities
## **How Does It Do It?**
- First and foremost: concurrency
- For each [[Component]], create a new [[ActionQueue]]
- After calling the [[Component#Update]] function, executes all of the [[Action]]s in the action queue

## **How Is It Structured?**
#### Method 1 - Immutable Slice To Other Components:
```Rust
#[derive(Debug, Clone, Copy)]
struct Item {
    field: f32
}

impl Item {
    fn update(&mut self, other_items: &[&Item], index: f32) {
        println!("{index} | {other_items:?}");
        self.field = index;
    }
}

#[derive(Debug)]
struct ItemCollection {
    all_items: Vec<Item>,
}

impl ItemCollection {
    fn update_all(&mut self) {
        for i in 0..self.all_items.len() {
            let (previous_items, next_items_and_current_item) = self.all_items.split_at_mut(i);
            let current_item = next_items_and_current_item.split_first_mut();

            if let Some((current_item, next_items)) = current_item {
                let chain = previous_items.iter().chain(next_items.iter()).collect::<Vec<_>>();
                current_item.update(&chain, i as f32);
            }
        }
    }
}
```
#### Method 2 - Mutable Slice To Other Components:
```Rust
#[derive(Debug, Clone, Copy)]
struct Item {
    field: f32,
}

impl Item {
    fn update(&mut self, other_items: &mut [&mut Item], index: f32) {
        other_items[0].field = 12.0 * index;
        println!("{index} | {other_items:?}");
        self.field = index;
    }
}

#[derive(Debug)]
struct ItemCollection {
    all_items: Vec<Item>,
}

impl ItemCollection {
    async fn update_all(&mut self) {
        let all_items_ref = &mut self.all_items;
        for i in 0..all_items_ref.len() {
            unsafe {
                let value = all_items_ref.split_at_mut(i);
                async_scoped::TokioScope::scope_and_collect(|scope| {
                    let proc = async move {
                        let (previous_items, next_items_and_current_item) = value;
                        let current_item = next_items_and_current_item.split_first_mut();
                        if let Some((current_item, next_items)) = current_item {
                            let mut chain = previous_items
                                .iter_mut()
                                .chain(next_items.iter_mut())
                                .collect::<Vec<_>>();
                            current_item.update(&mut chain, i as f32);
                        }
                    };

                    scope.spawn(proc);
                }).await;
            }
        }
    }
}
```
#### Method 3 - Slice Of Tokio Mutices:
- Does not work because they expect static lifetimes