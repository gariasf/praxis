use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
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
    VulkanLibrary,
};

struct TestContext {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    uniform_layout: Arc<DescriptorSetLayout>,
    combined_layout: Arc<DescriptorSetLayout>,
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

    let (device, _queues) = Device::new(
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

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        StandardDescriptorSetAllocatorCreateInfo::default(),
    ));

    // Create a simple descriptor set layout with one uniform buffer
    let uniform_layout = DescriptorSetLayout::new(
        device.clone(),
        DescriptorSetLayoutCreateInfo {
            bindings: [(
                0,
                DescriptorSetLayoutBinding {
                    stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    )
    .expect("Failed to create descriptor set layout");

    // Create a combined layout with multiple descriptors (typical for materials)
    let combined_layout = DescriptorSetLayout::new(
        device.clone(),
        DescriptorSetLayoutCreateInfo {
            bindings: [
                (
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                    },
                ),
                (
                    1,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(
                            DescriptorType::CombinedImageSampler,
                        )
                    },
                ),
                (
                    2,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(
                            DescriptorType::CombinedImageSampler,
                        )
                    },
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    )
    .expect("Failed to create combined descriptor set layout");

    TestContext {
        device,
        memory_allocator,
        descriptor_set_allocator,
        uniform_layout,
        combined_layout,
    }
}

fn create_uniform_buffer(
    memory_allocator: Arc<StandardMemoryAllocator>,
    size: usize,
) -> Subbuffer<[u8]> {
    Buffer::new_slice::<u8>(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        size as u64,
    )
    .expect("Failed to create uniform buffer")
}

fn bench_single_descriptor_allocation(c: &mut Criterion) {
    let ctx = create_test_context();
    let buffer = create_uniform_buffer(ctx.memory_allocator.clone(), 256);

    c.bench_function("descriptor_set_single_allocation", |b| {
        b.iter(|| {
            let descriptor_set = DescriptorSet::new(
                ctx.descriptor_set_allocator.clone(),
                ctx.uniform_layout.clone(),
                [WriteDescriptorSet::buffer(0, buffer.clone())],
                [],
            )
            .expect("Failed to create descriptor set");
            black_box(descriptor_set);
        });
    });
}

fn bench_batch_descriptor_allocation(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_set_batch_allocation");

    for batch_size in [10, 50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                let buffers: Vec<_> = (0..batch_size)
                    .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
                    .collect();

                b.iter(|| {
                    let descriptor_sets: Vec<_> = buffers
                        .iter()
                        .map(|buffer| {
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer.clone())],
                                [],
                            )
                            .expect("Failed to create descriptor set")
                        })
                        .collect();
                    black_box(descriptor_sets);
                });
            },
        );
    }

    group.finish();
}

fn bench_descriptor_reuse_vs_recreation(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_set_reuse_vs_recreation");

    let buffer = create_uniform_buffer(ctx.memory_allocator.clone(), 256);

    // Benchmark recreating descriptor sets each frame
    group.bench_function("recreate_every_frame", |b| {
        b.iter(|| {
            let descriptor_set = DescriptorSet::new(
                ctx.descriptor_set_allocator.clone(),
                ctx.uniform_layout.clone(),
                [WriteDescriptorSet::buffer(0, buffer.clone())],
                [],
            )
            .expect("Failed to create descriptor set");
            black_box(descriptor_set);
        });
    });

    // Benchmark reusing descriptor sets (simulated by just using the same set)
    group.bench_function("reuse_existing", |b| {
        let descriptor_set = DescriptorSet::new(
            ctx.descriptor_set_allocator.clone(),
            ctx.uniform_layout.clone(),
            [WriteDescriptorSet::buffer(0, buffer.clone())],
            [],
        )
        .expect("Failed to create descriptor set");

        b.iter(|| {
            black_box(&descriptor_set);
        });
    });

    group.finish();
}

fn bench_descriptor_pooling_patterns(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_pooling_patterns");

    // Pattern 1: Per-frame allocation (typical without pooling)
    group.bench_function("per_frame_allocation_pattern", |b| {
        b.iter(|| {
            let buffers: Vec<_> = (0..100)
                .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
                .collect();

            let descriptor_sets: Vec<_> = buffers
                .iter()
                .map(|buffer| {
                    DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.uniform_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer.clone())],
                        [],
                    )
                    .expect("Failed to create descriptor set")
                })
                .collect();

            black_box(descriptor_sets);
        });
    });

    // Pattern 2: Material-based pooling (shared descriptor sets per material)
    group.bench_function("material_pooling_pattern", |b| {
        // Pre-create 10 materials (descriptor sets)
        let material_sets: Vec<_> = (0..10)
            .map(|_| {
                let buffer = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
                DescriptorSet::new(
                    ctx.descriptor_set_allocator.clone(),
                    ctx.uniform_layout.clone(),
                    [WriteDescriptorSet::buffer(0, buffer)],
                    [],
                )
                .expect("Failed to create descriptor set")
            })
            .collect();

        b.iter(|| {
            // Simulate 100 objects using 10 materials (10 objects per material)
            for material in &material_sets {
                for _ in 0..10 {
                    black_box(material);
                }
            }
        });
    });

    group.finish();
}

fn bench_allocator_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("descriptor_allocator_configurations");

    // Test different allocator strategies
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
        .next()
        .expect("No device available");

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .position(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("Failed to find graphics queue family") as u32;

    let (device, _queues) = Device::new(
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

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    // Default allocator configuration
    group.bench_function("default_allocator_config", |b| {
        let allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo::default(),
        ));

        let layout = DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    0,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::VERTEX,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .expect("Failed to create layout");

        let buffer = create_uniform_buffer(memory_allocator.clone(), 256);

        b.iter(|| {
            let descriptor_set = DescriptorSet::new(
                allocator.clone(),
                layout.clone(),
                [WriteDescriptorSet::buffer(0, buffer.clone())],
                [],
            )
            .expect("Failed to create descriptor set");
            black_box(descriptor_set);
        });
    });

    group.finish();
}

fn bench_descriptor_write_patterns(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_write_patterns");

    // Single buffer write
    group.bench_function("single_buffer_write", |b| {
        let buffer = create_uniform_buffer(ctx.memory_allocator.clone(), 256);

        b.iter(|| {
            let descriptor_set = DescriptorSet::new(
                ctx.descriptor_set_allocator.clone(),
                ctx.uniform_layout.clone(),
                [WriteDescriptorSet::buffer(0, buffer.clone())],
                [],
            )
            .expect("Failed to create descriptor set");
            black_box(descriptor_set);
        });
    });

    // Multiple buffer writes (simulating transform + material data)
    group.bench_function("multiple_buffer_writes", |b| {
        let buffer1 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
        let buffer2 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
        let buffer3 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);

        // Create layout with 3 uniform buffers
        let multi_layout = DescriptorSetLayout::new(
            ctx.device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: [
                    (
                        0,
                        DescriptorSetLayoutBinding {
                            stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                            ..DescriptorSetLayoutBinding::descriptor_type(
                                DescriptorType::UniformBuffer,
                            )
                        },
                    ),
                    (
                        1,
                        DescriptorSetLayoutBinding {
                            stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                            ..DescriptorSetLayoutBinding::descriptor_type(
                                DescriptorType::UniformBuffer,
                            )
                        },
                    ),
                    (
                        2,
                        DescriptorSetLayoutBinding {
                            stages: ShaderStages::FRAGMENT,
                            ..DescriptorSetLayoutBinding::descriptor_type(
                                DescriptorType::UniformBuffer,
                            )
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .expect("Failed to create multi layout");

        b.iter(|| {
            let descriptor_set = DescriptorSet::new(
                ctx.descriptor_set_allocator.clone(),
                multi_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, buffer1.clone()),
                    WriteDescriptorSet::buffer(1, buffer2.clone()),
                    WriteDescriptorSet::buffer(2, buffer3.clone()),
                ],
                [],
            )
            .expect("Failed to create descriptor set");
            black_box(descriptor_set);
        });
    });

    group.finish();
}

fn bench_frame_by_frame_allocation(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("frame_by_frame_allocation");

    // Simulate a typical frame with varying object counts
    for object_count in [10, 50, 100, 200, 500] {
        group.throughput(Throughput::Elements(object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                b.iter(|| {
                    // Simulate per-frame descriptor set allocation for each object
                    let descriptor_sets: Vec<_> = (0..object_count)
                        .map(|_| {
                            let buffer = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer)],
                                [],
                            )
                            .expect("Failed to create descriptor set")
                        })
                        .collect();
                    black_box(descriptor_sets);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_descriptor_allocation,
    bench_batch_descriptor_allocation,
    bench_descriptor_reuse_vs_recreation,
    bench_descriptor_pooling_patterns,
    bench_allocator_configurations,
    bench_descriptor_write_patterns,
    bench_frame_by_frame_allocation,
);
criterion_main!(benches);
