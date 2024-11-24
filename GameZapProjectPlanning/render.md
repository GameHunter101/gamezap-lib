## **What Is It?**
- The function that gets run on the main thread during the main loop
	- Not async
	- DO NOT INTERRUPT THE RENDER
- Dictates the frame rate
- Draws all the elements
![[life_of_a_frame_wgpu.svg\|600]]
## **How Does It Do It?**
- Make a `RenderPass`
- Set the pipelines
- Set material bind groups
- Set component bind groups
## **What About UI?**
- All [[UI]] components should be tracked by the [[Scene]]
- Separate render pass for the UI
## **Code**
```Rust
pub struct Scene {
	...
	pipelines: Vec<Pipeline>,
	materials: Vec<Material>,
	components: HashMap<u32, Component>,
	...
}

impl Scene {
	pub fn render(&self, device: &wgpu::device, queue: &wgpu::Queue) {
		let mut encoder = device.create_command_encoder(...);
		{
			let mut render_pass = encoder.begin_render_pass(...);
			for pipeline in &pipelines {
				render_pass.set_pipeline(pipeline.render_pipeline);
				let material_indices = pipeline.corresponding_materials();
				for material_id in &material_indices {
					let material = &self.materials[material_id];
					material.set_bind_groups(render_pass);
					self.render_material_components(&device, &queue, &material, render_pass);
				}
			}
		}
	}

	pub fn render_material_components(&self, device: &wgpu::device, queue: &wgpu::Queue, material: &Material, render_pass: &mut wgpu::RenderPass) {
		let components_indices = material.corresponding_indices();
		for component_id in &component_indices {
			let component = self.components.get(component_id).expect(&format!("The component {component_id} was deleted but its materials were not updated"));
			component.render(&device, &queue, render_pass);
		}
	}
}
```