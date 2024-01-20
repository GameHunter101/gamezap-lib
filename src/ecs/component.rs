pub trait ComponentSystem {
    fn initialize(&mut self);
    fn update(&mut self);
}
