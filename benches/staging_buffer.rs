use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

struct TestContext {
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
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

    TestContext {
        device,
        queue,
        memory_allocator,
        command_buffer_allocator,
    }
}

fn create_staging_buffer(ctx: &TestContext, size: u64) -> Subbuffer<[u8]> {
    Buffer::new_slice::<u8>(
        ctx.memory_allocator.clone(),
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

fn create_device_buffer(ctx: &TestContext, size: u64) -> Subbuffer<[u8]> {
    Buffer::new_slice::<u8>(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST | BufferUsage::UNIFORM_BUFFER,
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

fn bench_staging_buffer_allocation(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("staging_buffer_allocation");

    for size in [256, 1024, 4096, 16384, 65536, 262144] {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let buffer = create_staging_buffer(&ctx, size as u64);
                black_box(buffer);
            });
        });
    }

    group.finish();
}

fn bench_staging_buffer_write(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("staging_buffer_write");

    for size in [256, 1024, 4096, 16384, 65536, 262144] {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let buffer = create_staging_buffer(&ctx, size as u64);
            let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

            b.iter(|| {
                let mut write_lock = buffer.write().expect("Failed to lock buffer");
                write_lock.copy_from_slice(&data);
                black_box(&write_lock);
            });
        });
    }

    group.finish();
}

fn bench_staging_to_device_copy(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("staging_to_device_copy");

    for size in [256, 1024, 4096, 16384, 65536, 262144] {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let staging = create_staging_buffer(&ctx, size as u64);
            let device_buf = create_device_buffer(&ctx, size as u64);

            // Fill staging buffer with data
            {
                let mut write_lock = staging.write().expect("Failed to lock buffer");
                let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
                write_lock.copy_from_slice(&data);
            }

            b.iter(|| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    ctx.command_buffer_allocator.clone(),
                    ctx.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .expect("Failed to create command buffer");

                builder
                    .copy_buffer(CopyBufferInfo::buffers(staging.clone(), device_buf.clone()))
                    .expect("Failed to record copy command");

                let command_buffer = builder.build().expect("Failed to build command buffer");

                let future = sync::now(ctx.device.clone())
                    .then_execute(ctx.queue.clone(), command_buffer)
                    .expect("Failed to execute command buffer")
                    .then_signal_fence_and_flush()
                    .expect("Failed to flush");

                future.wait(None).expect("Failed to wait for completion");
                black_box(&future);
            });
        });
    }

    group.finish();
}

fn bench_persistent_staging_buffer(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("persistent_staging_buffer");

    // Benchmark reusing a single staging buffer vs creating new ones each time
    let persistent_staging = create_staging_buffer(&ctx, 65536);

    group.bench_function("reuse_persistent_buffer", |b| {
        let device_buf = create_device_buffer(&ctx, 4096);
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();

        b.iter(|| {
            // Write to persistent staging buffer
            {
                let mut write_lock = persistent_staging.write().expect("Failed to lock buffer");
                write_lock[..4096].copy_from_slice(&data);
            }

            let mut builder = AutoCommandBufferBuilder::primary(
                ctx.command_buffer_allocator.clone(),
                ctx.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect("Failed to create command buffer");

            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    persistent_staging.clone(),
                    device_buf.clone(),
                ))
                .expect("Failed to record copy command");

            let command_buffer = builder.build().expect("Failed to build command buffer");

            let future = sync::now(ctx.device.clone())
                .then_execute(ctx.queue.clone(), command_buffer)
                .expect("Failed to execute command buffer")
                .then_signal_fence_and_flush()
                .expect("Failed to flush");

            future.wait(None).expect("Failed to wait for completion");
            black_box(&future);
        });
    });

    group.bench_function("create_new_buffer_each_time", |b| {
        let device_buf = create_device_buffer(&ctx, 4096);
        let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();

        b.iter(|| {
            let staging = create_staging_buffer(&ctx, 4096);

            // Write to new staging buffer
            {
                let mut write_lock = staging.write().expect("Failed to lock buffer");
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
                .expect("Failed to record copy command");

            let command_buffer = builder.build().expect("Failed to build command buffer");

            let future = sync::now(ctx.device.clone())
                .then_execute(ctx.queue.clone(), command_buffer)
                .expect("Failed to execute command buffer")
                .then_signal_fence_and_flush()
                .expect("Failed to flush");

            future.wait(None).expect("Failed to wait for completion");
            black_box(&future);
        });
    });

    group.finish();
}

fn bench_batch_staging_upload(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("batch_staging_upload");

    for batch_size in [1, 5, 10, 50, 100] {
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                let staging_buffers: Vec<_> = (0..batch_size)
                    .map(|_| create_staging_buffer(&ctx, 4096))
                    .collect();

                let device_buffers: Vec<_> = (0..batch_size)
                    .map(|_| create_device_buffer(&ctx, 4096))
                    .collect();

                // Fill staging buffers
                for staging in &staging_buffers {
                    let mut write_lock = staging.write().expect("Failed to lock buffer");
                    let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();
                    write_lock.copy_from_slice(&data);
                }

                b.iter(|| {
                    let mut builder = AutoCommandBufferBuilder::primary(
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create command buffer");

                    // Batch all copies into a single command buffer
                    for (staging, device_buf) in staging_buffers.iter().zip(device_buffers.iter()) {
                        builder
                            .copy_buffer(CopyBufferInfo::buffers(
                                staging.clone(),
                                device_buf.clone(),
                            ))
                            .expect("Failed to record copy command");
                    }

                    let command_buffer = builder.build().expect("Failed to build command buffer");

                    let future = sync::now(ctx.device.clone())
                        .then_execute(ctx.queue.clone(), command_buffer)
                        .expect("Failed to execute command buffer")
                        .then_signal_fence_and_flush()
                        .expect("Failed to flush");

                    future.wait(None).expect("Failed to wait for completion");
                    black_box(&future);
                });
            },
        );
    }

    group.finish();
}

fn bench_ring_buffer_staging(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("ring_buffer_staging");

    // Simulate a ring buffer pattern with 3 frames in flight
    let frames_in_flight = 3;
    let frame_size = 65536u64;
    let total_size = frame_size * frames_in_flight;

    let ring_buffer = create_staging_buffer(&ctx, total_size);
    let device_buf = create_device_buffer(&ctx, frame_size);

    group.bench_function("ring_buffer_write_pattern", |b| {
        let mut current_frame = 0usize;
        let data: Vec<u8> = (0..frame_size).map(|i| (i % 256) as u8).collect();

        b.iter(|| {
            let frame_offset = current_frame * frame_size as usize;

            // Write to current frame region
            {
                let mut write_lock = ring_buffer.write().expect("Failed to lock buffer");
                let frame_slice = &mut write_lock[frame_offset..frame_offset + frame_size as usize];
                frame_slice.copy_from_slice(&data);
            }

            // Simulate copy (we'd normally copy a sub-region, but for simplicity copy the whole thing)
            let mut builder = AutoCommandBufferBuilder::primary(
                ctx.command_buffer_allocator.clone(),
                ctx.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect("Failed to create command buffer");

            builder
                .copy_buffer(CopyBufferInfo::buffers(
                    ring_buffer.clone(),
                    device_buf.clone(),
                ))
                .expect("Failed to record copy command");

            let command_buffer = builder.build().expect("Failed to build command buffer");

            let future = sync::now(ctx.device.clone())
                .then_execute(ctx.queue.clone(), command_buffer)
                .expect("Failed to execute command buffer")
                .then_signal_fence_and_flush()
                .expect("Failed to flush");

            future.wait(None).expect("Failed to wait for completion");

            // Advance to next frame
            current_frame = (current_frame + 1) % frames_in_flight as usize;

            black_box(&future);
        });
    });

    group.finish();
}

fn bench_direct_buffer_write_vs_staging(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("direct_write_vs_staging");

    let data: Vec<u8> = (0..4096).map(|i| (i % 256) as u8).collect();

    // Direct write to host-visible device buffer
    group.bench_function("direct_host_write", |b| {
        let host_buffer = Buffer::new_slice::<u8>(
            ctx.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            4096,
        )
        .expect("Failed to create host buffer");

        b.iter(|| {
            let mut write_lock = host_buffer.write().expect("Failed to lock buffer");
            write_lock.copy_from_slice(&data);
            black_box(&write_lock);
        });
    });

    // Staging buffer with copy to device-local memory
    group.bench_function("staging_with_copy", |b| {
        let staging = create_staging_buffer(&ctx, 4096);
        let device_buf = create_device_buffer(&ctx, 4096);

        b.iter(|| {
            // Write to staging
            {
                let mut write_lock = staging.write().expect("Failed to lock buffer");
                write_lock.copy_from_slice(&data);
            }

            // Copy to device
            let mut builder = AutoCommandBufferBuilder::primary(
                ctx.command_buffer_allocator.clone(),
                ctx.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect("Failed to create command buffer");

            builder
                .copy_buffer(CopyBufferInfo::buffers(staging.clone(), device_buf.clone()))
                .expect("Failed to record copy command");

            let command_buffer = builder.build().expect("Failed to build command buffer");

            let future = sync::now(ctx.device.clone())
                .then_execute(ctx.queue.clone(), command_buffer)
                .expect("Failed to execute command buffer")
                .then_signal_fence_and_flush()
                .expect("Failed to flush");

            future.wait(None).expect("Failed to wait for completion");
            black_box(&future);
        });
    });

    group.finish();
}

fn bench_staging_buffer_sizes(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("staging_buffer_size_impact");

    // Test how different staging buffer sizes impact performance
    // when uploading the same amount of data
    let upload_size = 16384usize;
    let data: Vec<u8> = (0..upload_size).map(|i| (i % 256) as u8).collect();

    for staging_size in [16384, 32768, 65536, 131072, 262144] {
        group.throughput(Throughput::Bytes(upload_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(staging_size),
            &staging_size,
            |b, &staging_size| {
                let staging = create_staging_buffer(&ctx, staging_size as u64);
                let device_buf = create_device_buffer(&ctx, upload_size as u64);

                b.iter(|| {
                    // Write to staging buffer (only use first upload_size bytes)
                    {
                        let mut write_lock = staging.write().expect("Failed to lock buffer");
                        write_lock[..upload_size].copy_from_slice(&data);
                    }

                    let mut builder = AutoCommandBufferBuilder::primary(
                        ctx.command_buffer_allocator.clone(),
                        ctx.queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create command buffer");

                    builder
                        .copy_buffer(CopyBufferInfo::buffers(staging.clone(), device_buf.clone()))
                        .expect("Failed to record copy command");

                    let command_buffer = builder.build().expect("Failed to build command buffer");

                    let future = sync::now(ctx.device.clone())
                        .then_execute(ctx.queue.clone(), command_buffer)
                        .expect("Failed to execute command buffer")
                        .then_signal_fence_and_flush()
                        .expect("Failed to flush");

                    future.wait(None).expect("Failed to wait for completion");
                    black_box(&future);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_staging_buffer_allocation,
    bench_staging_buffer_write,
    bench_staging_to_device_copy,
    bench_persistent_staging_buffer,
    bench_batch_staging_upload,
    bench_ring_buffer_staging,
    bench_direct_buffer_write_vs_staging,
    bench_staging_buffer_sizes,
);
criterion_main!(benches);
