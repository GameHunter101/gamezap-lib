use std::fmt::Debug;

use super::scene::Scene;

pub trait Action<'a>: Debug {
    fn execute(&'a mut self, scene: &'a mut Scene<'a>);
}

#[derive(Debug)]
pub struct ActionQueue<'a> {
    actions: Vec<Box<dyn Action<'a>>>,
}

/* impl<'a: 'b, 'b> ActionQueue<'a, 'b> {
    pub fn execute_events(self, current_scene: &'b mut Scene<'b>) {
        for mut action in self.actions.into_iter() {
            action.execute(current_scene);
        }
    }
} */
