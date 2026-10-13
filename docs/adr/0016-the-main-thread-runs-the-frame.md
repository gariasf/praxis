# The main thread runs the frame, a compute pool lends it workers

The main thread runs winit, input, the simulation schedule, prepare and command recording, then submits and presents; a compute pool lends it workers inside the frame, and no stage runs a frame behind another. This is the lowest Input latency of the options surveyed (`docs/research/job-systems-and-frame-threading.md`, option A), needs no hand-off of the surface texture between threads, which matters because wgpu-hal 30.0.1's Metal acquire sends `-[NSView window]` (main-thread-only for Apple) on the calling thread ([surface.rs 354](https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/metal/surface.rs#L354-L375), [24](https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/metal/surface.rs#L24-L35)), and builds nothing before measurement asks for it. Accepted 2026-10-12 ([#20](https://github.com/gariasf/praxis/issues/20)). Planned; today the schedule runs without a pool because bevy_ecs's `multi_threaded` feature is off.

## Considered options

- A render thread one frame behind gives each stage its own 6 ms and costs about one more frame of Input latency, a main-thread acquire hand-off on Metal, and a second copy of per-frame state. It becomes the next step when the trigger below fires.
- A main thread for the OS only adds an input channel and a third thread on top of the render thread, for the same CPU headroom.
- A job graph without a render thread overlaps 2 to 3 stages, which risks the 2-frame Input latency budget (ADR 0013), and may need a task library beyond bevy_tasks.

## Consequences

- Frame data (ADR 0004) is an owned, `Send` value with no borrow into the ECS World, so moving rendering to its own thread is a channel send. Trigger: a Reference scene's main-thread critical path exceeds 6 ms while the pool is saturated, measured, not predicted.
- bevy_ecs's `multi_threaded` feature is on, for `par_iter` and scoped pool tasks, while the schedule keeps `ExecutorKind::SingleThreaded`: the multi-threaded executor costs 0.4 to 1.2 µs per system per run on the M4 Pro and pays off only for heavy, non-conflicting systems. It switches when a Baseline shows the gain.
- The `ComputeTaskPool` is created explicitly with `available_parallelism() − 1` workers, since the main thread joins scopes; `TaskPool::default` would put 13 threads on 12 cores. No QoS class is set: it made no measured difference.
- Asset reads and decoding run on a separate IO pool outside the compute pool. They write upload bytes into staging memory the renderer hands out; how that memory is managed is GPU memory and residency's decision ([#21](https://github.com/gariasf/praxis/issues/21)).
- Only the main thread submits, presents or polls the wgpu `Queue` and `Device`, so map and work-done callbacks run at a known point of the frame. Workers record `CommandEncoder`s and return `CommandBuffer`s; they push no error scopes, so their validation errors reach the uncaptured-error handler.
- `desired_maximum_frame_latency` stays 2. Per-frame data the GPU reads is written through `Queue::write_*`, which stages it and orders the copy before the next submit, or into a ring of mapped buffers whose slot is reused only after its `map_async` callback fires; wgpu completes a map only after the last submission using the buffer finishes, and refuses to submit work that uses a mapped buffer, so the ring decides whether the CPU waits, not whether reads are safe.
- Lua (ADR 0012) runs in one state on the main thread, with mlua's `send` feature off: no lock per call, no `Send + Sync` bound on userdata. Lua handler time counts against the 6 ms critical path.
- Recording 100k draws, one per Instance, has a 90th percentile of 5.1 to 6.9 ms at every thread count on the M4 Pro under wgpu 29.0.1 (11.5 to 15.5% more per draw on 30.0.1), which breaks a 6 ms stage here or in a render thread alike; the main pass's draw count is the visibility strategy's question ([#22](https://github.com/gariasf/praxis/issues/22)).
