use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo,
    },
    descriptor_set::{
        allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo},
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        DescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    shader::ShaderStages,
    sync::{self, GpuFuture},
    VulkanLibrary,
};

struct GraphicsContext {
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    material_layout: Arc<DescriptorSetLayout>,
    transform_layout: Arc<DescriptorSetLayout>,
}

impl GraphicsContext {
    fn new() -> Self {
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
            .filter(|p| p.properties().device_type == PhysicalDeviceType::DiscreteGpu)
            .next()
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

        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo::default(),
        ));

        // Material descriptor layout (uniform buffer)
        let material_layout = DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .expect("Failed to create material layout");

        // Transform descriptor layout (uniform buffer with dynamic offset)
        let transform_layout = DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::VERTEX,
                        ..DescriptorSetLayoutBinding::descriptor_type(
                            DescriptorType::UniformBufferDynamic,
                        )
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .expect("Failed to create transform layout");

        Self {
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            material_layout,
            transform_layout,
        }
    }

    fn create_uniform_buffer(&self, size: u64) -> Subbuffer<[u8]> {
        Buffer::new_slice::<u8>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            size,
        )
        .expect("Failed to create uniform buffer")
    }

    fn create_staging_buffer(&self, size: u64) -> Subbuffer<[u8]> {
        Buffer::new_slice::<u8>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            size,
        )
        .expect("Failed to create staging buffer")
    }

    fn create_device_buffer(&self, size: u64) -> Subbuffer<[u8]> {
        Buffer::new_slice::<u8>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            size,
        )
        .expect("Failed to create device buffer")
    }
}

fn bench_complete_frame_render_pattern(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("complete_frame_render_pattern");

    // Simulate a complete frame with object count varying
    for object_count in [10, 50, 100, 200] {
        group.throughput(Throughput::Elements(object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                b.iter(|| {
                    // Phase 1: Upload per-object data via staging buffer
                    let staging = ctx.create_staging_buffer((object_count * 256) as u64);
                    {
                        let mut write_lock = staging.write().expect("Failed to lock staging");
                        for i in 0..object_count {
                            let offset = i * 256;
                            let data: Vec<u8> = (0..256).map(|j| ((i + j) % 256) as u8).collect();
                            write_lock[offset..offset + 256].copy_from_slice(&data);
                        }
                    }

                    let device_buffer = ctx.create_device_buffer((object_count * 256) as u64);

                    // Phase 2: Copy staging to device
                    let mut builder = AutoCommandBufferBuilder::primary(
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create command buffer");

                    builder
                        .copy_buffer(CopyBufferInfo::buffers(staging, device_buffer.clone()))
                        .expect("Failed to record copy");

                    let command_buffer = builder.build().expect("Failed to build command buffer");

                    let future = sync::now(ctx.device.clone())
                        .then_execute(ctx.queue.clone(), command_buffer)
                        .expect("Failed to execute")
                        .then_signal_fence_and_flush()
                        .expect("Failed to flush");

                    future.wait(None).expect("Failed to wait");

                    // Phase 3: Create descriptor sets for materials (simulating material batching)
                    let materials_count = (object_count / 10).max(1); // 10 objects per material
                    let _material_sets: Vec<_> = (0..materials_count)
                        .map(|_| {
                            let material_buffer = ctx.create_uniform_buffer(256);
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.material_layout.clone(),
                                [WriteDescriptorSet::buffer(0, material_buffer)],
                                [],
                            )
                            .expect("Failed to create material descriptor set")
                        })
                        .collect();

                    black_box((_material_sets, device_buffer));
                });
            },
        );
    }

    group.finish();
}

fn bench_material_batching_optimization(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("material_batching_optimization");

    let object_count = 100;

    // Scenario 1: No batching - each object gets its own descriptor set
    group.bench_function("no_batching", |b| {
        b.iter(|| {
            let descriptor_sets: Vec<_> = (0..object_count)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();
            black_box(descriptor_sets);
        });
    });

    // Scenario 2: Material batching - 10 materials shared by 100 objects
    group.bench_function("with_batching_10_materials", |b| {
        b.iter(|| {
            // Create 10 material descriptor sets
            let material_sets: Vec<_> = (0..10)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Simulate reusing each material for 10 objects
            for material in &material_sets {
                for _ in 0..10 {
                    black_box(material);
                }
            }

            black_box(material_sets);
        });
    });

    // Scenario 3: Aggressive batching - 5 materials shared by 100 objects
    group.bench_function("with_batching_5_materials", |b| {
        b.iter(|| {
            let material_sets: Vec<_> = (0..5)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Simulate reusing each material for 20 objects
            for material in &material_sets {
                for _ in 0..20 {
                    black_box(material);
                }
            }

            black_box(material_sets);
        });
    });

    group.finish();
}

fn bench_dynamic_uniform_buffer_pattern(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("dynamic_uniform_buffer_pattern");

    // Simulate the dynamic uniform buffer pattern used in the engine
    for object_count in [10, 50, 100, 200, 500] {
        group.throughput(Throughput::Elements(object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                // Get minimum alignment
                let min_alignment = ctx
                    .device
                    .physical_device()
                    .properties()
                    .min_uniform_buffer_offset_alignment
                    .as_devicesize() as usize;

                let object_size = 64; // Size of a 4x4 matrix
                let aligned_size =
                    ((object_size + min_alignment - 1) / min_alignment) * min_alignment;

                // Create a large buffer for all objects
                let total_size = aligned_size * object_count;
                let dynamic_buffer = ctx.create_uniform_buffer(total_size as u64);

                b.iter(|| {
                    // Write all object matrices
                    {
                        let mut write_lock = dynamic_buffer.write().expect("Failed to lock buffer");
                        for i in 0..object_count {
                            let offset = i * aligned_size;
                            let matrix_data: Vec<u8> =
                                (0..64).map(|j| ((i + j) % 256) as u8).collect();
                            write_lock[offset..offset + 64].copy_from_slice(&matrix_data);
                        }
                    }

                    // Create a single descriptor set with dynamic offsets
                    let descriptor_set = DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.transform_layout.clone(),
                        [WriteDescriptorSet::buffer(0, dynamic_buffer.clone())],
                        [],
                    )
                    .expect("Failed to create descriptor set");

                    // Simulate binding with different dynamic offsets for each object
                    for i in 0..object_count {
                        let _dynamic_offset = (i * aligned_size) as u32;
                        black_box(&descriptor_set);
                    }

                    black_box(descriptor_set);
                });
            },
        );
    }

    group.finish();
}

fn bench_descriptor_set_caching(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("descriptor_set_caching");

    // Simulate caching descriptor sets per material across frames
    let material_count = 10;
    let frames = 60; // Simulate 60 frames

    group.bench_function("no_caching", |b| {
        b.iter(|| {
            // Recreate descriptor sets every frame
            for _ in 0..frames {
                let _descriptor_sets: Vec<_> = (0..material_count)
                    .map(|_| {
                        let buffer = ctx.create_uniform_buffer(256);
                        DescriptorSet::new(
                            ctx.descriptor_set_allocator.clone(),
                            ctx.material_layout.clone(),
                            [WriteDescriptorSet::buffer(0, buffer)],
                            [],
                        )
                        .expect("Failed to create descriptor set")
                    })
                    .collect();
                black_box(_descriptor_sets);
            }
        });
    });

    group.bench_function("with_caching", |b| {
        b.iter(|| {
            // Create descriptor sets once, reuse across frames
            let descriptor_sets: Vec<_> = (0..material_count)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Reuse across frames
            for _ in 0..frames {
                for set in &descriptor_sets {
                    black_box(set);
                }
            }

            black_box(descriptor_sets);
        });
    });

    group.finish();
}

fn bench_staging_buffer_pooling(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("staging_buffer_pooling");

    let upload_size = 16384;
    let frames = 10;

    group.bench_function("no_pooling", |b| {
        b.iter(|| {
            for _ in 0..frames {
                let staging = ctx.create_staging_buffer(upload_size);
                let device_buf = ctx.create_device_buffer(upload_size);

                {
                    let mut write_lock = staging.write().expect("Failed to lock");
                    let data: Vec<u8> = (0..upload_size).map(|i| (i % 256) as u8).collect();
                    write_lock.copy_from_slice(&data);
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                builder
                    .copy_buffer(CopyBufferInfo::buffers(staging, device_buf.clone()))
                    .expect("Failed to record copy");

                let command_buffer = builder.build().expect("Failed to build");

                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait");
                black_box(device_buf);
            }
        });
    });

    group.bench_function("with_pooling", |b| {
        b.iter(|| {
            // Create a pool of 3 staging buffers (ring buffer pattern)
            let pool: Vec<_> = (0..3)
                .map(|_| ctx.create_staging_buffer(upload_size))
                .collect();

            for frame in 0..frames {
                let staging = &pool[frame % 3];
                let device_buf = ctx.create_device_buffer(upload_size);

                {
                    let mut write_lock = staging.write().expect("Failed to lock");
                    let data: Vec<u8> = (0..upload_size).map(|i| (i % 256) as u8).collect();
                    write_lock.copy_from_slice(&data);
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                builder
                    .copy_buffer(CopyBufferInfo::buffers(staging.clone(), device_buf.clone()))
                    .expect("Failed to record copy");

                let command_buffer = builder.build().expect("Failed to build");

                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait");
                black_box(device_buf);
            }

            black_box(pool);
        });
    });

    group.finish();
}

fn bench_integrated_optimization_scenarios(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("integrated_optimization_scenarios");

    let object_count = 100;

    // Baseline: Current approach (no optimizations)
    group.bench_function("baseline_current_approach", |b| {
        b.iter(|| {
            // Per-object descriptor sets
            let descriptor_sets: Vec<_> = (0..object_count)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Per-object staging uploads
            for _ in 0..object_count {
                let staging = ctx.create_staging_buffer(1024);
                let device_buf = ctx.create_device_buffer(1024);

                {
                    let mut write_lock = staging.write().expect("Failed to lock");
                    let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
                    write_lock.copy_from_slice(&data);
                }

                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                builder
                    .copy_buffer(CopyBufferInfo::buffers(staging, device_buf.clone()))
                    .expect("Failed to record copy");

                let command_buffer = builder.build().expect("Failed to build");

                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait");
                black_box(device_buf);
            }

            black_box(descriptor_sets);
        });
    });

    // Optimized: Material batching + staging buffer pooling
    group.bench_function("optimized_batching_and_pooling", |b| {
        b.iter(|| {
            // Material batching: 10 materials for 100 objects
            let material_sets: Vec<_> = (0..10)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Staging buffer pooling: single large staging buffer
            let staging = ctx.create_staging_buffer((object_count * 1024) as u64);
            let device_buf = ctx.create_device_buffer((object_count * 1024) as u64);

            {
                let mut write_lock = staging.write().expect("Failed to lock");
                for i in 0..object_count {
                    let offset = i * 1024;
                    let data: Vec<u8> = (0..1024usize).map(|j| ((i + j) % 256) as u8).collect();
                    write_lock[offset..offset + 1024].copy_from_slice(&data);
                }
            }

            let mut builder = AutoCommandBufferBuilder::primary(
                ctx.command_buffer_allocator.clone(),
                ctx.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect("Failed to create command buffer");

            builder
                .copy_buffer(CopyBufferInfo::buffers(staging, device_buf.clone()))
                .expect("Failed to record copy");

            let command_buffer = builder.build().expect("Failed to build");

            let future = sync::now(ctx.device.clone())
                .then_execute(ctx.queue.clone(), command_buffer)
                .expect("Failed to execute")
                .then_signal_fence_and_flush()
                .expect("Failed to flush");

            future.wait(None).expect("Failed to wait");

            black_box((material_sets, device_buf));
        });
    });

    group.finish();
}

fn bench_multi_draw_indirect(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("multi_draw_indirect_rendering");

    // Test configurations
    let test_configs = [
        (500, 20, "500_objects_20_materials"),
        (750, 20, "750_objects_20_materials"),
        (1000, 20, "1000_objects_20_materials"),
    ];

    for (object_count, material_count, name) in test_configs {
        // Baseline: Traditional individual draw calls
        group.bench_function(&format!("{}_traditional", name), |b| {
            // Pre-allocate materials and descriptor sets
            let material_sets: Vec<_> = (0..material_count)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            b.iter(|| {
                let mut draw_call_count = 0u32;
                let start = std::time::Instant::now();

                // Simulate individual draw calls for each object
                for i in 0..object_count {
                    let material_idx = i % material_count;

                    // Simulate binding material descriptor set (represents CPU overhead)
                    let _material_set = &material_sets[material_idx];

                    // Simulate draw call (this represents the CPU-side call overhead)
                    draw_call_count += 1;

                    // Prevent compiler optimization
                    black_box(draw_call_count);
                }

                let cpu_time = start.elapsed();

                black_box((draw_call_count, cpu_time));

                draw_call_count
            });
        });

        // Optimized: Multi-draw indirect with material batching
        group.bench_function(&format!("{}_multi_draw_indirect", name), |b| {
            use vulkano::command_buffer::DrawIndexedIndirectCommand;

            // Pre-allocate materials (same as traditional)
            let material_sets: Vec<_> = (0..material_count)
                .map(|_| {
                    let buffer = ctx.create_uniform_buffer(256);
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.material_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer)],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            // Create indirect draw buffer
            let indirect_commands: Vec<DrawIndexedIndirectCommand> = (0..object_count)
                .map(|_| DrawIndexedIndirectCommand {
                    index_count: 36, // Simple cube
                    instance_count: 1,
                    first_index: 0,
                    vertex_offset: 0,
                    first_instance: 0,
                })
                .collect();

            let indirect_buffer = Buffer::from_iter(
                ctx.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDIRECT_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indirect_commands,
            )
            .expect("Failed to create indirect buffer");

            b.iter(|| {
                let mut draw_call_count = 0u32;
                let start = std::time::Instant::now();

                // Group objects by material to create batches
                let mut i = 0;
                while i < object_count {
                    let material_idx = i % material_count;
                    let _material_set = &material_sets[material_idx];

                    // Find consecutive objects with the same material
                    let batch_start = i;
                    while i < object_count && (i % material_count) == material_idx {
                        i += 1;
                    }
                    let batch_size = i - batch_start;

                    // Single multi-draw indirect call for entire batch
                    draw_call_count += 1;

                    // Simulate indirect buffer slice access
                    let _batch_slice = &indirect_buffer;

                    black_box((draw_call_count, batch_size));
                }

                let cpu_time = start.elapsed();

                black_box((draw_call_count, cpu_time, &indirect_buffer));

                draw_call_count
            });
        });
    }

    group.finish();
}

fn bench_draw_call_reduction_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_call_reduction_analysis");

    // Analyze draw call reduction with different material distributions
    let object_count = 1000;

    for material_count in [10, 20, 50, 100] {
        group.bench_function(&format!("analysis_{}_materials", material_count), |b| {
            b.iter(|| {
                // Traditional rendering: one draw call per object
                let traditional_draw_calls = object_count;

                // Multi-draw indirect: one draw call per material batch
                // With objects sorted by material, we get one batch per material
                let multi_draw_calls = material_count;

                // Calculate reduction factor
                let reduction_factor = traditional_draw_calls as f32 / multi_draw_calls as f32;

                black_box((traditional_draw_calls, multi_draw_calls, reduction_factor));

                reduction_factor
            });
        });
    }

    group.finish();
}

fn bench_indirect_buffer_build_cost(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("indirect_buffer_build_cost");

    for object_count in [500, 750, 1000] {
        group.bench_function(&format!("{}_objects", object_count), |b| {
            b.iter(|| {
                use vulkano::command_buffer::DrawIndexedIndirectCommand;

                // Simulate building indirect draw commands
                let commands: Vec<DrawIndexedIndirectCommand> = (0..object_count)
                    .map(|i| DrawIndexedIndirectCommand {
                        index_count: 36,
                        instance_count: 1,
                        first_index: (i * 36) as u32,
                        vertex_offset: 0,
                        first_instance: i as u32,
                    })
                    .collect();

                // Upload to GPU buffer
                let buffer = Buffer::from_iter(
                    ctx.memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::INDIRECT_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    commands,
                )
                .expect("Failed to create buffer");

                black_box(buffer);
            });
        });
    }

    group.finish();
}

fn bench_material_batching_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_batching_overhead");

    let object_count = 1000;
    let material_count = 20;

    // Benchmark sorting objects by material
    group.bench_function("sort_by_material", |b| {
        // Create object list with material IDs
        let objects: Vec<(usize, u32)> = (0..object_count)
            .map(|i| (i, (i % material_count) as u32))
            .collect();

        b.iter(|| {
            let mut sorted_objects = objects.clone();
            sorted_objects.sort_by_key(|(_, material_id)| *material_id);
            black_box(sorted_objects);
        });
    });

    // Benchmark grouping consecutive objects by material
    group.bench_function("group_by_material", |b| {
        // Pre-sorted object list
        let mut objects: Vec<(usize, u32)> = (0..object_count)
            .map(|i| (i, (i % material_count) as u32))
            .collect();
        objects.sort_by_key(|(_, material_id)| *material_id);

        b.iter(|| {
            let mut batches = Vec::new();
            let mut i = 0;

            while i < objects.len() {
                let material_id = objects[i].1;
                let batch_start = i;

                // Find consecutive objects with same material
                while i < objects.len() && objects[i].1 == material_id {
                    i += 1;
                }

                let batch_size = i - batch_start;
                batches.push((material_id, batch_start, batch_size));
            }

            black_box(batches);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_complete_frame_render_pattern,
    bench_material_batching_optimization,
    bench_dynamic_uniform_buffer_pattern,
    bench_descriptor_set_caching,
    bench_staging_buffer_pooling,
    bench_integrated_optimization_scenarios,
    bench_multi_draw_indirect,
    bench_draw_call_reduction_analysis,
    bench_indirect_buffer_build_cost,
    bench_material_batching_overhead,
);
criterion_main!(benches);
