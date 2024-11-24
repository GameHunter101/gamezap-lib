#### Design
- Async as possible
	- Dynamic structured concurrency
	- [Article 1](https://shahbhat.medium.com/structured-concurrency-in-modern-programming-languages-part-i-e7cdb25ff172)
	- [Article 2](https://emschwartz.me/async-rust-can-be-a-pleasure-to-work-with-without-send-sync-static)
- [[ECS Specifications]]
- Separate `update`, `render`, and `initialize` methods
	- `async` so that `update` doesn't block `render`
		- Maybe send `initialize` and `update` to different threads?
	- `render` should happen on the main thread
	- Able to cancel all `async` calls on program close
#### Global state
- Engine initialization time
- Time since last frame
	- Delta time
- Frames executed last second
	- FPS
- Last frame initialization time
- Window reference
- All scenes
- Active scene
- Window size
- Render mask
- Is cursor visible
#### Functions
- `main_loop`
	- Spawns threads for [[initialize]] and [[update]] functions
	- Calls [[render]] on the main thread