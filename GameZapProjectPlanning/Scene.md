## **What Is It?**
- A representation of all of the engine's responsibilities
- A collection of everything that needs to happen every frame
## **What Does It Store**
- Hash maps of:
	- Component ID to enum of workload future or its output
	- Entity ID to [[Entity]]
	- Pipeline ID to vector of material IDs corresponding to each pipeline
- Vectors of:
	- [[Component|Components]]
	- [[Material|Materials]]
	- Component IDs of all UI components
	- [[Pipeline|Pipelines]] and [[Material]] ID vectors
- One-off variables:
	- Active camera Component ID
	- Count of total entities created
	- Text state for text creation
## **What Does It Do?**
- Executes the [[initialize]], [[update]], and [[render]] functions on every [[Component]] appropriately