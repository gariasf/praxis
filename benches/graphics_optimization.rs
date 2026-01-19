use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use praxis_graphics::gpu_culling::{GpuCullingManager, GpuDrawCommand, GpuMeshData};
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_procedural::compression::{CompressionFormat, CompressionQuality, TextureCompressor};
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

fn bench_gpu_vs_cpu_lod_selection(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("gpu_vs_cpu_lod_selection");
    group.sample_size(50);

    // Test configurations: 100, 1000, 10000, 100000 objects
    for object_count in [100, 1000, 10000, 100000] {
        group.throughput(Throughput::Elements(object_count as u64));

        // ===== CPU LOD Selection Benchmark =====
        group.bench_function(BenchmarkId::new("cpu_lod_selection", object_count), |b| {
            use praxis_graphics::lod::{LodGroup, LodLevel};

            // Setup camera position
            let camera_position = Vec3::new(0.0, 0.0, 50.0);

            // Setup LOD group with 3 levels
            let lod_group = LodGroup::new(vec![
                LodLevel::new("high", 0.0, 10.0),    // 0-10 units
                LodLevel::new("medium", 10.0, 25.0), // 10-25 units
                LodLevel::new("low", 25.0, 100.0),   // 25-100 units
            ]);

            // Setup test objects in a grid
            let grid_size = (object_count as f32).cbrt().ceil() as usize;
            let spacing = 10.0;
            let mut object_positions = Vec::with_capacity(object_count);

            for i in 0..object_count {
                let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let y = (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let z = ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;
                object_positions.push(Vec3::new(x, y, z));
            }

            b.iter(|| {
                let start = std::time::Instant::now();
                let mut selected_lods = Vec::with_capacity(object_count);

                // CPU LOD selection - calculate distance and select LOD for each object
                for object_position in &object_positions {
                    // Calculate squared distance
                    let delta = *object_position - camera_position;
                    let distance_squared = delta.length_squared();

                    // Select LOD level
                    let selected_level = lod_group.select_lod_level(distance_squared);
                    selected_lods.push(selected_level);
                }

                let cpu_time = start.elapsed();
                black_box((selected_lods.len(), cpu_time))
            });
        });

        // ===== GPU LOD Selection Benchmark =====
        group.bench_function(BenchmarkId::new("gpu_lod_selection", object_count), |b| {
            use praxis_graphics::lod::{GpuLodLevel, GpuLodSelector, GpuObjectData};

            // Setup GPU LOD selector
            let mut lod_selector = GpuLodSelector::new(
                ctx.device.clone(),
                ctx.memory_allocator.clone(),
                Arc::new(StandardDescriptorSetAllocator::new(
                    ctx.device.clone(),
                    Default::default(),
                )),
            )
            .expect("Failed to create GPU LOD selector");

            // Setup camera position
            let camera_position = Vec3::new(0.0, 0.0, 50.0);

            // Setup test objects in a grid (same distribution as CPU test)
            let grid_size = (object_count as f32).cbrt().ceil() as usize;
            let spacing = 10.0;
            let mut objects = Vec::with_capacity(object_count);
            let mut lod_levels = Vec::with_capacity(object_count * 3);

            for i in 0..object_count {
                let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let y = (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let z = ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;

                let model = Mat4::from_translation(Vec3::new(x, y, z));
                let bounding_sphere = [0.0, 0.0, 0.0, 1.0]; // Center at origin, radius 1.0

                objects.push(GpuObjectData::new(
                    model,
                    bounding_sphere,
                    i as u32,
                    3,
                    (i * 3) as u32,
                ));
            }

            // Define LOD levels (3 levels per object)
            for _ in 0..object_count {
                lod_levels.push(GpuLodLevel {
                    mesh_id: 0,
                    min_distance_sq: 0.0,
                    max_distance_sq: 100.0, // 10^2
                    padding: 0,
                });
                lod_levels.push(GpuLodLevel {
                    mesh_id: 1,
                    min_distance_sq: 100.0,
                    max_distance_sq: 625.0, // 25^2
                    padding: 0,
                });
                lod_levels.push(GpuLodLevel {
                    mesh_id: 2,
                    min_distance_sq: 625.0,
                    max_distance_sq: 10000.0, // 100^2
                    padding: 0,
                });
            }

            // Prepare buffers once
            lod_selector
                .prepare_frame(&objects, &lod_levels)
                .expect("Failed to prepare GPU LOD frame");

            b.iter(|| {
                let start = std::time::Instant::now();

                // Create command buffer for GPU LOD selection
                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                // Dispatch GPU LOD selection compute shader
                lod_selector
                    .dispatch_lod_selection(&mut builder, camera_position, 0.0, true)
                    .expect("Failed to dispatch GPU LOD selection");

                let command_buffer = builder.build().expect("Failed to build command buffer");

                // Submit and wait
                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait for GPU");

                let total_time = start.elapsed();

                // Read back results for verification (optional, but ensures work completed)
                let selected_count = lod_selector
                    .read_selected_lods()
                    .expect("Failed to read selected LODs")
                    .len();

                black_box((selected_count, total_time))
            });
        });

        // ===== CPU Overhead Only Benchmark (GPU LOD) =====
        group.bench_function(
            BenchmarkId::new("gpu_lod_cpu_overhead_only", object_count),
            |b| {
                use praxis_graphics::lod::{GpuLodLevel, GpuLodSelector, GpuObjectData};

                // This benchmark measures only CPU-side overhead of preparing and dispatching
                // GPU LOD selection, excluding actual GPU execution time

                let mut lod_selector = GpuLodSelector::new(
                    ctx.device.clone(),
                    ctx.memory_allocator.clone(),
                    Arc::new(StandardDescriptorSetAllocator::new(
                        ctx.device.clone(),
                        Default::default(),
                    )),
                )
                .expect("Failed to create GPU LOD selector");

                let camera_position = Vec3::new(0.0, 0.0, 50.0);

                let grid_size = (object_count as f32).cbrt().ceil() as usize;
                let spacing = 10.0;
                let mut objects = Vec::with_capacity(object_count);
                let mut lod_levels = Vec::with_capacity(object_count * 3);

                for i in 0..object_count {
                    let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                    let y =
                        (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                    let z =
                        ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;

                    let model = Mat4::from_translation(Vec3::new(x, y, z));
                    objects.push(GpuObjectData::new(
                        model,
                        [0.0, 0.0, 0.0, 1.0], // Center at origin, radius 1.0
                        i as u32,
                        3,
                        (i * 3) as u32,
                    ));
                }

                for _ in 0..object_count {
                    lod_levels.push(GpuLodLevel {
                        mesh_id: 0,
                        min_distance_sq: 0.0,
                        max_distance_sq: 100.0,
                        padding: 0,
                    });
                    lod_levels.push(GpuLodLevel {
                        mesh_id: 1,
                        min_distance_sq: 100.0,
                        max_distance_sq: 625.0,
                        padding: 0,
                    });
                    lod_levels.push(GpuLodLevel {
                        mesh_id: 2,
                        min_distance_sq: 625.0,
                        max_distance_sq: 10000.0,
                        padding: 0,
                    });
                }

                lod_selector
                    .prepare_frame(&objects, &lod_levels)
                    .expect("Failed to prepare GPU LOD frame");

                b.iter(|| {
                    // Measure only CPU-side preparation and dispatch (not GPU execution)
                    let start = std::time::Instant::now();

                    let mut builder = AutoCommandBufferBuilder::primary(
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create command buffer");

                    lod_selector
                        .dispatch_lod_selection(&mut builder, camera_position, 0.0, true)
                        .expect("Failed to dispatch GPU LOD selection");

                    let _command_buffer = builder.build().expect("Failed to build command buffer");

                    let cpu_time = start.elapsed();

                    // Note: We don't submit/wait here, just measuring CPU overhead
                    black_box(cpu_time)
                });
            },
        );
    }

    group.finish();
}

fn bench_texture_compression(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("texture_compression");
    group.sample_size(20);

    // Test configurations: 256x256, 512x512, 1024x1024
    let texture_sizes = [(256, "256x256"), (512, "512x512"), (1024, "1024x1024")];

    for (size, name) in texture_sizes {
        let width = size;
        let height = size;
        let pixel_count = (width * height) as u64;

        group.throughput(Throughput::Elements(pixel_count));

        // Create test texture data (RGBA8)
        let uncompressed_size = (width * height * 4) as usize;
        let test_data: Vec<u8> = (0..uncompressed_size).map(|i| (i % 256) as u8).collect();

        // ===== BC7 Compression Benchmark (Fast Quality) =====
        group.bench_function(BenchmarkId::new("bc7_fast", name), |b| {
            let mut compressor = TextureCompressor::new(
                ctx.device.clone(),
                ctx.queue.clone(),
                ctx.memory_allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.descriptor_set_allocator.clone(),
            );

            b.iter(|| {
                let start = std::time::Instant::now();

                let compressed = compressor
                    .compress(
                        &test_data,
                        width,
                        height,
                        CompressionFormat::BC7,
                        CompressionQuality::Fast,
                    )
                    .expect("BC7 compression failed");

                let gpu_time = start.elapsed();

                // Verify compression metrics
                let compression_ratio = compressed.compression_ratio();
                let vram_savings = compressed.vram_savings();
                let vram_savings_percent = (vram_savings as f32 / uncompressed_size as f32) * 100.0;

                black_box((
                    compressed.data.len(),
                    compression_ratio,
                    vram_savings,
                    vram_savings_percent,
                    gpu_time,
                ))
            });
        });

        // ===== BC7 Compression Benchmark (High Quality) =====
        group.bench_function(BenchmarkId::new("bc7_high", name), |b| {
            let mut compressor = TextureCompressor::new(
                ctx.device.clone(),
                ctx.queue.clone(),
                ctx.memory_allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.descriptor_set_allocator.clone(),
            );

            b.iter(|| {
                let start = std::time::Instant::now();

                let compressed = compressor
                    .compress(
                        &test_data,
                        width,
                        height,
                        CompressionFormat::BC7,
                        CompressionQuality::High,
                    )
                    .expect("BC7 compression failed");

                let gpu_time = start.elapsed();

                let compression_ratio = compressed.compression_ratio();
                let vram_savings = compressed.vram_savings();
                let vram_savings_percent = (vram_savings as f32 / uncompressed_size as f32) * 100.0;

                black_box((
                    compressed.data.len(),
                    compression_ratio,
                    vram_savings,
                    vram_savings_percent,
                    gpu_time,
                ))
            });
        });

        // ===== BC5 Compression Benchmark (Fast Quality) =====
        group.bench_function(BenchmarkId::new("bc5_fast", name), |b| {
            let mut compressor = TextureCompressor::new(
                ctx.device.clone(),
                ctx.queue.clone(),
                ctx.memory_allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.descriptor_set_allocator.clone(),
            );

            b.iter(|| {
                let start = std::time::Instant::now();

                let compressed = compressor
                    .compress(
                        &test_data,
                        width,
                        height,
                        CompressionFormat::BC5,
                        CompressionQuality::Fast,
                    )
                    .expect("BC5 compression failed");

                let gpu_time = start.elapsed();

                let compression_ratio = compressed.compression_ratio();
                let vram_savings = compressed.vram_savings();
                let vram_savings_percent = (vram_savings as f32 / uncompressed_size as f32) * 100.0;

                black_box((
                    compressed.data.len(),
                    compression_ratio,
                    vram_savings,
                    vram_savings_percent,
                    gpu_time,
                ))
            });
        });

        // ===== BC5 Compression Benchmark (High Quality) =====
        group.bench_function(BenchmarkId::new("bc5_high", name), |b| {
            let mut compressor = TextureCompressor::new(
                ctx.device.clone(),
                ctx.queue.clone(),
                ctx.memory_allocator.clone(),
                ctx.command_buffer_allocator.clone(),
                ctx.descriptor_set_allocator.clone(),
            );

            b.iter(|| {
                let start = std::time::Instant::now();

                let compressed = compressor
                    .compress(
                        &test_data,
                        width,
                        height,
                        CompressionFormat::BC5,
                        CompressionQuality::High,
                    )
                    .expect("BC5 compression failed");

                let gpu_time = start.elapsed();

                let compression_ratio = compressed.compression_ratio();
                let vram_savings = compressed.vram_savings();
                let vram_savings_percent = (vram_savings as f32 / uncompressed_size as f32) * 100.0;

                black_box((
                    compressed.data.len(),
                    compression_ratio,
                    vram_savings,
                    vram_savings_percent,
                    gpu_time,
                ))
            });
        });

        // ===== Compression Metrics Analysis =====
        group.bench_function(BenchmarkId::new("metrics_analysis", name), |b| {
            b.iter(|| {
                // Calculate theoretical metrics
                let uncompressed_bytes = (width * height * 4) as usize;
                let blocks_width = width / 4;
                let blocks_height = height / 4;
                let num_blocks = blocks_width * blocks_height;
                let compressed_bytes = (num_blocks * 16) as usize;

                let compression_ratio = uncompressed_bytes as f32 / compressed_bytes as f32;
                let vram_savings = uncompressed_bytes - compressed_bytes;
                let vram_savings_percent =
                    (vram_savings as f32 / uncompressed_bytes as f32) * 100.0;

                // Verify 4:1 compression and 75% VRAM reduction
                assert_eq!(compression_ratio, 4.0, "Should achieve 4:1 compression");
                assert_eq!(
                    vram_savings_percent, 75.0,
                    "Should achieve 75% VRAM reduction"
                );

                black_box((
                    uncompressed_bytes,
                    compressed_bytes,
                    compression_ratio,
                    vram_savings,
                    vram_savings_percent,
                ))
            });
        });
    }

    group.finish();
}

fn bench_gpu_vs_cpu_culling(c: &mut Criterion) {
    let ctx = GraphicsContext::new();
    let mut group = c.benchmark_group("gpu_vs_cpu_culling");
    group.sample_size(50);

    // Test configurations: 1000, 5000, 10000 objects
    for object_count in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(object_count as u64));

        // ===== CPU Frustum Culling Benchmark =====
        group.bench_function(BenchmarkId::new("cpu_culling", object_count), |b| {
            // Setup camera and frustum
            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 50.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            );
            let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
            let view_proj = proj * view;

            // Extract frustum planes
            let m = view_proj.to_cols_array_2d();
            let normalize_plane = |plane: [f32; 4]| -> [f32; 4] {
                let length =
                    (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
                [
                    plane[0] / length,
                    plane[1] / length,
                    plane[2] / length,
                    plane[3] / length,
                ]
            };

            let frustum_planes = [
                normalize_plane([
                    m[0][3] + m[0][0],
                    m[1][3] + m[1][0],
                    m[2][3] + m[2][0],
                    m[3][3] + m[3][0],
                ]), // left
                normalize_plane([
                    m[0][3] - m[0][0],
                    m[1][3] - m[1][0],
                    m[2][3] - m[2][0],
                    m[3][3] - m[3][0],
                ]), // right
                normalize_plane([
                    m[0][3] + m[0][1],
                    m[1][3] + m[1][1],
                    m[2][3] + m[2][1],
                    m[3][3] + m[3][1],
                ]), // bottom
                normalize_plane([
                    m[0][3] - m[0][1],
                    m[1][3] - m[1][1],
                    m[2][3] - m[2][1],
                    m[3][3] - m[3][1],
                ]), // top
                normalize_plane([
                    m[0][3] + m[0][2],
                    m[1][3] + m[1][2],
                    m[2][3] + m[2][2],
                    m[3][3] + m[3][2],
                ]), // near
                normalize_plane([
                    m[0][3] - m[0][2],
                    m[1][3] - m[1][2],
                    m[2][3] - m[2][2],
                    m[3][3] - m[3][2],
                ]), // far
            ];

            // Setup test objects in a grid (approximately 50% will be visible)
            let grid_size = (object_count as f32).cbrt().ceil() as usize;
            let spacing = 10.0;
            let mut objects = Vec::with_capacity(object_count);

            for i in 0..object_count {
                let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let y = (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let z = ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;

                objects.push((
                    Vec3::new(x, y, z), // center
                    2.0,                // radius
                ));
            }

            b.iter(|| {
                let start = std::time::Instant::now();
                let mut visible_count = 0;

                // CPU frustum culling - test each object sequentially
                for (center, radius) in &objects {
                    let mut is_visible = true;

                    // Test sphere against all 6 frustum planes
                    for plane in &frustum_planes {
                        let distance = plane[0] * center.x
                            + plane[1] * center.y
                            + plane[2] * center.z
                            + plane[3];
                        if distance < -radius {
                            is_visible = false;
                            break;
                        }
                    }

                    if is_visible {
                        visible_count += 1;
                    }
                }

                let cpu_time = start.elapsed();
                black_box((visible_count, cpu_time))
            });
        });

        // ===== GPU Compute Culling Benchmark =====
        group.bench_function(BenchmarkId::new("gpu_culling", object_count), |b| {
            // Setup GPU culling manager
            let mut culling_manager = GpuCullingManager::new(
                ctx.device.clone(),
                ctx.memory_allocator.clone(),
                Arc::new(StandardDescriptorSetAllocator::new(
                    ctx.device.clone(),
                    Default::default(),
                )),
            )
            .expect("Failed to create GPU culling manager");

            // Setup camera and frustum
            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 50.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            );
            let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
            let view_proj = proj * view;
            let camera_pos = Vec3::new(0.0, 0.0, 50.0);

            // Extract frustum planes
            let frustum_planes = praxis_graphics::gpu_culling::extract_frustum_planes(view_proj);

            // Setup test objects in a grid (same distribution as CPU test)
            let grid_size = (object_count as f32).cbrt().ceil() as usize;
            let spacing = 10.0;
            let mut draw_commands = Vec::with_capacity(object_count);
            let mut mesh_data = Vec::with_capacity(object_count);

            for i in 0..object_count {
                let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let y = (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let z = ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;

                let model = Mat4::from_translation(Vec3::new(x, y, z));
                let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 2.0); // local space sphere

                draw_commands.push(GpuDrawCommand::new(model, bounding_sphere, i as u32, 0));

                mesh_data.push(GpuMeshData {
                    index_count: 36,
                    first_index: 0,
                    vertex_offset: 0,
                    _padding: 0,
                });
            }

            // Prepare buffers once
            culling_manager
                .prepare_frame(&draw_commands, &mesh_data)
                .expect("Failed to prepare GPU culling frame");

            b.iter(|| {
                let start = std::time::Instant::now();

                // Create command buffer for GPU culling
                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                // Dispatch GPU culling compute shader
                culling_manager
                    .dispatch_culling(&mut builder, view_proj, frustum_planes, camera_pos)
                    .expect("Failed to dispatch GPU culling");

                let command_buffer = builder.build().expect("Failed to build command buffer");

                // Submit and wait (in real usage, this would be async)
                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait for GPU");

                let cpu_time = start.elapsed();

                // Read back visible count for verification
                let visible_count = culling_manager
                    .read_visible_count()
                    .expect("Failed to read visible count");

                black_box((visible_count, cpu_time))
            });
        });

        // ===== CPU Overhead Only Benchmark (no actual culling) =====
        group.bench_function(BenchmarkId::new("cpu_overhead_only", object_count), |b| {
            // This benchmark measures the CPU-side overhead of preparing and dispatching
            // GPU culling, excluding GPU execution time

            let mut culling_manager = GpuCullingManager::new(
                ctx.device.clone(),
                ctx.memory_allocator.clone(),
                Arc::new(StandardDescriptorSetAllocator::new(
                    ctx.device.clone(),
                    Default::default(),
                )),
            )
            .expect("Failed to create GPU culling manager");

            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 50.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            );
            let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
            let view_proj = proj * view;
            let camera_pos = Vec3::new(0.0, 0.0, 50.0);
            let frustum_planes = praxis_graphics::gpu_culling::extract_frustum_planes(view_proj);

            let grid_size = (object_count as f32).cbrt().ceil() as usize;
            let spacing = 10.0;
            let mut draw_commands = Vec::with_capacity(object_count);
            let mut mesh_data = Vec::with_capacity(object_count);

            for i in 0..object_count {
                let x = ((i % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let y = (((i / grid_size) % grid_size) as f32 - grid_size as f32 / 2.0) * spacing;
                let z = ((i / (grid_size * grid_size)) as f32 - grid_size as f32 / 2.0) * spacing;

                let model = Mat4::from_translation(Vec3::new(x, y, z));
                let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 2.0);

                draw_commands.push(GpuDrawCommand::new(model, bounding_sphere, i as u32, 0));
                mesh_data.push(GpuMeshData {
                    index_count: 36,
                    first_index: 0,
                    vertex_offset: 0,
                    _padding: 0,
                });
            }

            culling_manager
                .prepare_frame(&draw_commands, &mesh_data)
                .expect("Failed to prepare GPU culling frame");

            b.iter(|| {
                // Measure only CPU-side preparation and dispatch (not GPU execution)
                let start = std::time::Instant::now();

                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                culling_manager
                    .dispatch_culling(&mut builder, view_proj, frustum_planes, camera_pos)
                    .expect("Failed to dispatch GPU culling");

                let _command_buffer = builder.build().expect("Failed to build command buffer");

                let cpu_time = start.elapsed();

                // Note: We don't submit/wait here, just measuring CPU overhead
                black_box(cpu_time)
            });
        });
    }

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
    bench_gpu_vs_cpu_lod_selection,
    bench_gpu_vs_cpu_culling,
    bench_texture_compression,
);
criterion_main!(benches);
