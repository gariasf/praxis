use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use praxis_graphics::mesh::{MeshData, MeshStreamingSystem};
use std::sync::Arc;
use std::time::{Duration, Instant};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    VulkanLibrary,
};

struct TestContext {
    allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
}

fn create_test_context() -> TestContext {
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            ..Default::default()
        },
    )
    .expect("Failed to create instance");

    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Failed to enumerate devices")
        .find(|p| p.properties().device_type == PhysicalDeviceType::DiscreteGpu)
        .or_else(|| {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate devices")
                .next()
        })
        .expect("No device available");

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .position(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("Failed to find graphics queue family") as u32;

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("Failed to create device");

    let queue = queues.next().expect("Failed to get queue");
    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device,
        Default::default(),
    ));

    TestContext {
        allocator,
        command_buffer_allocator,
        queue,
    }
}

fn create_mesh_data(vertex_count: usize) -> MeshData {
    let mut positions = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let t = i as f32 / vertex_count as f32;
        positions.push([
            t * 10.0,
            (t * std::f32::consts::TAU).sin(),
            (t * std::f32::consts::TAU).cos(),
        ]);
        colors.push([t, 1.0 - t, 0.5]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([t, t]);
    }

    let mut indices = Vec::with_capacity((vertex_count / 3) * 3);
    for i in (0..vertex_count - 2).step_by(3) {
        indices.push(i as u16);
        indices.push((i + 1) as u16);
        indices.push((i + 2) as u16);
    }

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: Some(uvs),
        tangents: None,
        indices,
    }
}

fn bench_mesh_upload(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("mesh_upload");

    for vertex_count in [100, 500, 1000, 5000, 10000, 50000] {
        let mesh_data = create_mesh_data(vertex_count);
        group.throughput(Throughput::Elements(vertex_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(vertex_count),
            &mesh_data,
            |b, mesh_data| {
                b.iter(|| {
                    let _gpu_mesh = mesh_data
                        .upload(
                            ctx.allocator.clone(),
                            ctx.command_buffer_allocator.clone(),
                            ctx.queue.clone(),
                        )
                        .expect("Failed to upload mesh");
                    black_box(_gpu_mesh);
                });
            },
        );
    }

    group.finish();
}

fn bench_mesh_upload_with_textures(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("mesh_upload_textured");

    for vertex_count in [1000, 5000, 10000] {
        let mesh_data = create_mesh_data(vertex_count);
        group.throughput(Throughput::Elements(vertex_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(vertex_count),
            &mesh_data,
            |b, mesh_data| {
                b.iter(|| {
                    let _gpu_mesh = mesh_data
                        .upload(
                            ctx.allocator.clone(),
                            ctx.command_buffer_allocator.clone(),
                            ctx.queue.clone(),
                        )
                        .expect("Failed to upload mesh");
                    black_box(_gpu_mesh);
                });
            },
        );
    }

    group.finish();
}

fn bench_primitive_generation_and_upload(c: &mut Criterion) {
    let ctx = create_test_context();

    c.bench_function("simple_triangle_generation_and_upload", |b| {
        b.iter(|| {
            let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
            let indices = vec![0, 1, 2];
            let mesh_data = MeshData::new(positions, indices);
            let _gpu_mesh = mesh_data
                .upload(
                    ctx.allocator.clone(),
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.clone(),
                )
                .expect("Failed to upload mesh");
            black_box(_gpu_mesh);
        });
    });

    c.bench_function("quad_generation_and_upload", |b| {
        b.iter(|| {
            let positions = vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ];
            let indices = vec![0, 1, 2, 0, 2, 3];
            let mesh_data = MeshData::new(positions, indices);
            let _gpu_mesh = mesh_data
                .upload(
                    ctx.allocator.clone(),
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.clone(),
                )
                .expect("Failed to upload mesh");
            black_box(_gpu_mesh);
        });
    });
}

fn bench_streaming_throughput(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("streaming_throughput");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    for mesh_count in [10, 50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(mesh_count as u64));

        group.bench_function(BenchmarkId::new("meshes_per_second", mesh_count), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::ZERO;

                for _ in 0..iters {
                    let mut streaming_system = MeshStreamingSystem::new(
                        ctx.allocator.clone(),
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.clone(),
                    );

                    // Create mesh data for all meshes
                    let mesh_data_list: Vec<MeshData> =
                        (0..mesh_count).map(|_| create_mesh_data(1000)).collect();

                    // Register all meshes
                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        streaming_system
                            .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                            .expect("Failed to register mesh");
                    }

                    let start = Instant::now();

                    // Request loading all meshes with priority
                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        streaming_system.request_load(
                            &format!("mesh_{}", i),
                            mesh_data.clone(),
                            100.0,
                        );
                    }

                    // Poll until all meshes are loaded
                    let mut loaded_count = 0;
                    let timeout = Duration::from_secs(30);
                    let poll_start = Instant::now();

                    while loaded_count < mesh_count {
                        streaming_system.update();
                        loaded_count = streaming_system.loaded_count();

                        if Instant::now() - poll_start > timeout {
                            panic!("Timeout waiting for meshes to load");
                        }

                        std::thread::sleep(Duration::from_millis(1));
                    }

                    total_duration += start.elapsed();

                    black_box(loaded_count);
                }

                total_duration
            });
        });
    }

    group.finish();
}

fn bench_streaming_priority_queue(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("streaming_priority_queue");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    for mesh_count in [100, 500, 1000, 2000] {
        group.throughput(Throughput::Elements(mesh_count as u64));

        group.bench_function(BenchmarkId::new("priority_processing", mesh_count), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::ZERO;

                for _ in 0..iters {
                    let mut streaming_system = MeshStreamingSystem::new(
                        ctx.allocator.clone(),
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.clone(),
                    );

                    let mesh_data_list: Vec<MeshData> =
                        (0..mesh_count).map(|_| create_mesh_data(500)).collect();

                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        streaming_system
                            .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                            .expect("Failed to register mesh");
                    }

                    let start = Instant::now();

                    // Submit with varying priorities (simulating distance-based priorities)
                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        let priority = 100.0 - (i as f32 * 0.1);
                        streaming_system.request_load(
                            &format!("mesh_{}", i),
                            mesh_data.clone(),
                            priority,
                        );
                    }

                    // Process until all loaded
                    let mut loaded_count = 0;
                    let timeout = Duration::from_secs(60);
                    let poll_start = Instant::now();

                    while loaded_count < mesh_count {
                        streaming_system.update();
                        loaded_count = streaming_system.loaded_count();

                        if Instant::now() - poll_start > timeout {
                            panic!("Timeout waiting for priority queue processing");
                        }

                        std::thread::sleep(Duration::from_millis(1));
                    }

                    total_duration += start.elapsed();

                    black_box(loaded_count);
                }

                total_duration
            });
        });
    }

    group.finish();
}

fn bench_streaming_background_thread_overhead(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("streaming_background_overhead");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark system initialization overhead
    group.bench_function("system_init_and_shutdown", |b| {
        b.iter(|| {
            let streaming_system = MeshStreamingSystem::new(
                ctx.allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.queue.clone(),
            );
            black_box(streaming_system);
            // System is dropped here, thread joins
        });
    });

    // Benchmark empty update overhead
    group.bench_function("empty_update_overhead", |b| {
        let mut streaming_system = MeshStreamingSystem::new(
            ctx.allocator.clone(),
            ctx.command_buffer_allocator.clone(),
            ctx.queue.clone(),
        );

        b.iter(|| {
            streaming_system.update();
            black_box(&streaming_system);
        });
    });

    // Benchmark registration overhead
    group.bench_function("registration_overhead_100_meshes", |b| {
        b.iter(|| {
            let mut streaming_system = MeshStreamingSystem::new(
                ctx.allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.queue.clone(),
            );

            let mesh_data = create_mesh_data(1000);

            for i in 0..100 {
                streaming_system
                    .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                    .expect("Failed to register mesh");
            }

            black_box(streaming_system);
        });
    });

    // Benchmark update overhead with many registered meshes
    group.bench_function("update_overhead_1000_meshes", |b| {
        let mut streaming_system = MeshStreamingSystem::new(
            ctx.allocator.clone(),
            ctx.command_buffer_allocator.clone(),
            ctx.queue.clone(),
        );

        let mesh_data = create_mesh_data(1000);

        for i in 0..1000 {
            streaming_system
                .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                .expect("Failed to register mesh");
        }

        b.iter(|| {
            streaming_system.update();
            black_box(&streaming_system);
        });
    });

    group.finish();
}

fn bench_streaming_non_blocking_behavior(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("streaming_non_blocking");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    // Verify main thread is not blocked while background loading happens
    group.bench_function("main_thread_responsiveness", |b| {
        b.iter_custom(|iters| {
            let mut total_blocking_time = Duration::ZERO;

            for _ in 0..iters {
                let mut streaming_system = MeshStreamingSystem::new(
                    ctx.allocator.clone(),
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.clone(),
                );

                let mesh_data_list: Vec<MeshData> =
                    (0..100).map(|_| create_mesh_data(1000)).collect();

                for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                    streaming_system
                        .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                        .expect("Failed to register mesh");
                }

                // Measure time taken by main thread operations
                let start = Instant::now();

                // Submit all load requests (should be non-blocking)
                for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                    streaming_system.request_load(&format!("mesh_{}", i), mesh_data.clone(), 100.0);
                }

                // Record time for submission (should be minimal)
                let submission_time = start.elapsed();

                // Now perform other work while background thread processes
                let work_start = Instant::now();
                let mut work_iterations = 0;

                // Simulate frame updates - should not block
                for _ in 0..100 {
                    streaming_system.update();
                    work_iterations += 1;

                    // Simulate other frame work
                    std::thread::sleep(Duration::from_micros(100));
                }

                let work_time = work_start.elapsed();

                // Total blocking should just be submission + update calls, not loading time
                total_blocking_time += submission_time + work_time;

                // Wait for completion
                while streaming_system.loaded_count() < 100 {
                    streaming_system.update();
                    std::thread::sleep(Duration::from_millis(1));
                }

                black_box(work_iterations);
            }

            total_blocking_time
        });
    });

    // Benchmark concurrent request/update pattern
    group.bench_function("concurrent_request_update_pattern", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let mut streaming_system = MeshStreamingSystem::new(
                    ctx.allocator.clone(),
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.clone(),
                );

                let mesh_data_list: Vec<MeshData> =
                    (0..200).map(|_| create_mesh_data(500)).collect();

                for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                    streaming_system
                        .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                        .expect("Failed to register mesh");
                }

                let start = Instant::now();

                // Interleave requests and updates
                for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                    streaming_system.request_load(&format!("mesh_{}", i), mesh_data.clone(), 100.0);

                    // Update every 10 requests
                    if i % 10 == 0 {
                        streaming_system.update();
                    }
                }

                // Final updates until complete
                while streaming_system.loaded_count() < 200 {
                    streaming_system.update();
                    std::thread::sleep(Duration::from_millis(1));
                }

                total_duration += start.elapsed();

                black_box(&streaming_system);
            }

            total_duration
        });
    });

    group.finish();
}

fn bench_streaming_large_scale(c: &mut Criterion) {
    let ctx = create_test_context();

    let mut group = c.benchmark_group("streaming_large_scale");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    // Test with 1000+ meshes to verify scalability
    for mesh_count in [1000, 2000, 5000] {
        group.throughput(Throughput::Elements(mesh_count as u64));

        group.bench_function(BenchmarkId::new("load_all", mesh_count), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::ZERO;

                for _ in 0..iters {
                    let mut streaming_system = MeshStreamingSystem::new(
                        ctx.allocator.clone(),
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.clone(),
                    );

                    let mesh_data_list: Vec<MeshData> =
                        (0..mesh_count).map(|_| create_mesh_data(300)).collect();

                    // Register all
                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        streaming_system
                            .register_mesh(format!("mesh_{}", i), mesh_data.clone())
                            .expect("Failed to register mesh");
                    }

                    let start = Instant::now();

                    // Request all with varying priorities
                    for (i, mesh_data) in mesh_data_list.iter().enumerate() {
                        let priority = 100.0 - (i as f32 % 100.0);
                        streaming_system.request_load(
                            &format!("mesh_{}", i),
                            mesh_data.clone(),
                            priority,
                        );
                    }

                    // Poll until complete
                    let timeout = Duration::from_secs(120);
                    let poll_start = Instant::now();

                    while streaming_system.loaded_count() < mesh_count {
                        streaming_system.update();

                        if Instant::now() - poll_start > timeout {
                            panic!(
                                "Timeout: loaded {}/{} meshes",
                                streaming_system.loaded_count(),
                                mesh_count
                            );
                        }

                        std::thread::sleep(Duration::from_millis(1));
                    }

                    total_duration += start.elapsed();

                    black_box(&streaming_system);
                }

                total_duration
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_mesh_upload,
    bench_mesh_upload_with_textures,
    bench_primitive_generation_and_upload,
    bench_streaming_throughput,
    bench_streaming_priority_queue,
    bench_streaming_background_thread_overhead,
    bench_streaming_non_blocking_behavior,
    bench_streaming_large_scale,
);
criterion_main!(benches);
