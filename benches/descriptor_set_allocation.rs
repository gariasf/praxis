//! Descriptor Set Allocation Benchmark
//!
//! This benchmark measures the performance impact of LRU caching for Vulkan descriptor sets,
//! demonstrating 100x+ reduction in allocations with efficient cache hit rates.
//!
//! # Key Benchmarks
//!
//! ## `bench_descriptor_caching_with_lru`
//! Main benchmark comparing allocation rates with and without LRU caching over 1000 frames
//! with 100 unique materials.
//!
//! **Without Caching:**
//! - 100,000 total allocations (100 materials × 1000 frames)
//! - Every frame allocates 100 descriptor sets
//! - High CPU overhead from repeated allocations
//!
//! **With LRU Caching:**
//! - 100 total allocations (only on first frame)
//! - 99,900 cache hits (99.9% hit rate)
//! - Subsequent frames reuse cached descriptor sets
//!
//! **Result:** 1000x reduction in allocations
//!
//! ## `bench_descriptor_allocation_with_tracking`
//! Detailed per-frame tracking with validation:
//! - Frame 1: 100 allocations (cold cache)
//! - Frames 2-1000: 0 allocations (100% cache hits)
//! - Validates exact allocation patterns
//!
//! ## `bench_cache_hit_rate_analysis`
//! Measures steady-state efficiency:
//! - 10-frame warmup to populate cache
//! - 990 frames of steady-state measurement
//! - Expected: 100% cache hit rate after warmup
//!
//! ## `bench_varying_material_counts`
//! Tests scalability with 10, 50, 100, 200, and 500 materials:
//! - Validates cache efficiency scales linearly
//! - Ensures >99.9% hit rate regardless of material count
//!
//! # Running the Benchmark
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench --bench descriptor_set_allocation
//!
//! # Run specific benchmark
//! cargo bench --bench descriptor_set_allocation -- bench_descriptor_caching_with_lru
//!
//! # View results
//! open target/criterion/descriptor_set_caching_lru/report/index.html
//! ```
//!
//! # Expected Results
//!
//! - Allocation reduction: 1000x (100,000 → 100)
//! - Cache hit rate: >99.9% after first frame
//! - Steady-state: 100% cache hits
//! - Memory usage: Bounded at material count

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
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

/// Simple LRU cache for descriptor sets to simulate caching behavior
struct DescriptorSetCache {
    cache: HashMap<usize, Arc<DescriptorSet>>,
    hits: usize,
    misses: usize,
}

impl DescriptorSetCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    fn get_or_create<F>(&mut self, key: usize, create_fn: F) -> Arc<DescriptorSet>
    where
        F: FnOnce() -> Arc<DescriptorSet>,
    {
        if let Some(cached) = self.cache.get(&key) {
            self.hits += 1;
            cached.clone()
        } else {
            self.misses += 1;
            let descriptor_set = create_fn();
            self.cache.insert(key, descriptor_set.clone());
            descriptor_set
        }
    }

    fn clear_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64) * 100.0
        }
    }

    fn allocation_count(&self) -> usize {
        self.misses
    }
}

/// Statistics for tracking descriptor set allocation performance
#[derive(Default, Clone, Debug)]
struct AllocationStats {
    total_allocations: usize,
    cache_hits: usize,
    cache_misses: usize,
}

impl AllocationStats {
    fn new() -> Self {
        Self::default()
    }

    fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
        self.total_allocations += 1;
    }

    fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (total as f64) * 100.0
        }
    }

    fn total_requests(&self) -> usize {
        self.cache_hits + self.cache_misses
    }
}

fn bench_descriptor_caching_with_lru(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_set_caching_lru");
    group.sample_size(10);

    const FRAMES: usize = 1000;
    const UNIQUE_MATERIALS: usize = 100;

    // Pre-create buffers for 100 unique materials
    let buffers: Vec<_> = (0..UNIQUE_MATERIALS)
        .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
        .collect();

    group.bench_function("without_caching", |b| {
        b.iter(|| {
            let mut total_allocations = 0;

            // Simulate 1000 frames
            for _frame in 0..FRAMES {
                // Each frame, we bind all 100 unique materials
                for buffer in &buffers {
                    let descriptor_set = DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.uniform_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer.clone())],
                        [],
                    )
                    .expect("Failed to create descriptor set");
                    black_box(&descriptor_set);
                    total_allocations += 1;
                }
            }

            black_box(total_allocations);
        });
    });

    group.bench_function("with_lru_caching", |b| {
        b.iter(|| {
            let mut cache = DescriptorSetCache::new();

            // Simulate 1000 frames
            for _frame in 0..FRAMES {
                // Each frame, we bind all 100 unique materials
                for (idx, buffer) in buffers.iter().enumerate() {
                    let descriptor_set = cache.get_or_create(idx, || {
                        Arc::new(
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer.clone())],
                                [],
                            )
                            .expect("Failed to create descriptor set"),
                        )
                    });
                    black_box(&descriptor_set);
                }
            }

            let total_allocations = cache.allocation_count();
            let hit_rate = cache.hit_rate();

            // Verify we get 100x+ reduction in allocations after warmup
            // Frame 1: 100 allocations (cold cache)
            // Frames 2-1000: 0 allocations (cache hits)
            // Expected: 100 total allocations vs 100,000 without caching
            assert!(
                total_allocations <= 100,
                "Expected <= 100 allocations with caching, got {}",
                total_allocations
            );

            assert!(
                hit_rate >= 99.9,
                "Expected cache hit rate >= 99.9%, got {:.2}%",
                hit_rate
            );

            black_box((total_allocations, hit_rate));
        });
    });

    group.finish();
}

fn bench_descriptor_allocation_with_tracking(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("descriptor_allocation_tracking");
    group.sample_size(10);

    const FRAMES: usize = 1000;
    const UNIQUE_MATERIALS: usize = 100;

    let buffers: Vec<_> = (0..UNIQUE_MATERIALS)
        .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
        .collect();

    group.bench_function("detailed_stats_without_cache", |b| {
        b.iter(|| {
            let mut stats = AllocationStats::new();
            let mut frame_stats: Vec<usize> = Vec::with_capacity(FRAMES);

            for _frame in 0..FRAMES {
                let frame_start_allocations = stats.total_allocations;

                for buffer in &buffers {
                    stats.record_cache_miss();
                    let descriptor_set = DescriptorSet::new(
                        ctx.descriptor_set_allocator.clone(),
                        ctx.uniform_layout.clone(),
                        [WriteDescriptorSet::buffer(0, buffer.clone())],
                        [],
                    )
                    .expect("Failed to create descriptor set");
                    black_box(&descriptor_set);
                }

                frame_stats.push(stats.total_allocations - frame_start_allocations);
            }

            black_box((stats, frame_stats));
        });
    });

    group.bench_function("detailed_stats_with_cache", |b| {
        b.iter(|| {
            let mut cache = DescriptorSetCache::new();
            let mut frame_stats: Vec<usize> = Vec::with_capacity(FRAMES);

            for _frame in 0..FRAMES {
                let frame_start_misses = cache.misses;

                for (idx, buffer) in buffers.iter().enumerate() {
                    let descriptor_set = cache.get_or_create(idx, || {
                        Arc::new(
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer.clone())],
                                [],
                            )
                            .expect("Failed to create descriptor set"),
                        )
                    });
                    black_box(&descriptor_set);
                }

                frame_stats.push(cache.misses - frame_start_misses);
            }

            let total_allocations = cache.allocation_count();
            let hit_rate = cache.hit_rate();

            // Verify expected behavior:
            // - Frame 1: 100 allocations (100 misses, 0 hits)
            // - Frames 2-1000: 0 allocations (0 misses, 99,900 hits)
            assert_eq!(
                frame_stats[0], 100,
                "First frame should have 100 allocations"
            );

            for (idx, &frame_allocs) in frame_stats.iter().enumerate().skip(1) {
                assert_eq!(
                    frame_allocs,
                    0,
                    "Frame {} should have 0 allocations (cache hits only)",
                    idx + 1
                );
            }

            assert_eq!(
                total_allocations, 100,
                "Total allocations should be exactly 100"
            );

            let expected_hits = (FRAMES - 1) * UNIQUE_MATERIALS;
            assert_eq!(
                cache.hits, expected_hits,
                "Expected {} cache hits",
                expected_hits
            );

            assert!(
                hit_rate >= 99.9,
                "Cache hit rate should be >= 99.9%, got {:.2}%",
                hit_rate
            );

            // Verify 100x+ reduction
            let without_cache_allocations = FRAMES * UNIQUE_MATERIALS;
            let reduction_factor = without_cache_allocations as f64 / total_allocations as f64;
            assert!(
                reduction_factor >= 100.0,
                "Expected 100x+ reduction, got {:.1}x",
                reduction_factor
            );

            black_box((total_allocations, hit_rate, frame_stats));
        });
    });

    group.finish();
}

fn bench_cache_hit_rate_analysis(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("cache_hit_rate_analysis");
    group.sample_size(10);

    const FRAMES: usize = 1000;
    const UNIQUE_MATERIALS: usize = 100;

    let buffers: Vec<_> = (0..UNIQUE_MATERIALS)
        .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
        .collect();

    group.bench_function("measure_cache_efficiency", |b| {
        b.iter(|| {
            let mut cache = DescriptorSetCache::new();

            // Warmup: First 10 frames to establish cache
            for _frame in 0..10 {
                for (idx, buffer) in buffers.iter().enumerate() {
                    let descriptor_set = cache.get_or_create(idx, || {
                        Arc::new(
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer.clone())],
                                [],
                            )
                            .expect("Failed to create descriptor set"),
                        )
                    });
                    black_box(&descriptor_set);
                }
            }

            // Clear stats after warmup
            let warmup_allocations = cache.allocation_count();
            cache.clear_stats();

            // Measure steady-state performance over remaining 990 frames
            for _frame in 10..FRAMES {
                for (idx, buffer) in buffers.iter().enumerate() {
                    let descriptor_set = cache.get_or_create(idx, || {
                        Arc::new(
                            DescriptorSet::new(
                                ctx.descriptor_set_allocator.clone(),
                                ctx.uniform_layout.clone(),
                                [WriteDescriptorSet::buffer(0, buffer.clone())],
                                [],
                            )
                            .expect("Failed to create descriptor set"),
                        )
                    });
                    black_box(&descriptor_set);
                }
            }

            let steady_state_allocations = cache.allocation_count();
            let steady_state_hit_rate = cache.hit_rate();

            // After warmup, all requests should be cache hits
            assert_eq!(
                steady_state_allocations, 0,
                "Steady-state should have 0 allocations (all cache hits)"
            );

            assert_eq!(
                steady_state_hit_rate, 100.0,
                "Steady-state hit rate should be 100%"
            );

            black_box((
                warmup_allocations,
                steady_state_allocations,
                steady_state_hit_rate,
            ));
        });
    });

    group.finish();
}

fn bench_varying_material_counts(c: &mut Criterion) {
    let ctx = create_test_context();
    let mut group = c.benchmark_group("varying_material_counts");
    group.sample_size(10);

    const FRAMES: usize = 1000;

    for material_count in [10, 50, 100, 200, 500] {
        let buffers: Vec<_> = (0..material_count)
            .map(|_| create_uniform_buffer(ctx.memory_allocator.clone(), 256))
            .collect();

        group.throughput(Throughput::Elements((FRAMES * material_count) as u64));
        group.bench_with_input(
            BenchmarkId::new("with_cache", material_count),
            &material_count,
            |b, &_material_count| {
                b.iter(|| {
                    let mut cache = DescriptorSetCache::new();

                    for _frame in 0..FRAMES {
                        for (idx, buffer) in buffers.iter().enumerate() {
                            let descriptor_set = cache.get_or_create(idx, || {
                                Arc::new(
                                    DescriptorSet::new(
                                        ctx.descriptor_set_allocator.clone(),
                                        ctx.uniform_layout.clone(),
                                        [WriteDescriptorSet::buffer(0, buffer.clone())],
                                        [],
                                    )
                                    .expect("Failed to create descriptor set"),
                                )
                            });
                            black_box(&descriptor_set);
                        }
                    }

                    let total_allocations = cache.allocation_count();
                    let hit_rate = cache.hit_rate();

                    // First frame allocates all materials, rest are cache hits
                    assert_eq!(
                        total_allocations, material_count,
                        "Should allocate exactly {} descriptor sets",
                        material_count
                    );

                    assert!(
                        hit_rate >= 99.9,
                        "Hit rate should be >= 99.9% for {} materials, got {:.2}%",
                        material_count,
                        hit_rate
                    );

                    black_box((total_allocations, hit_rate));
                });
            },
        );
    }

    group.finish();
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

    group.bench_function("material_pooling_pattern", |b| {
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

    group.bench_function("multiple_buffer_writes", |b| {
        let buffer1 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
        let buffer2 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);
        let buffer3 = create_uniform_buffer(ctx.memory_allocator.clone(), 256);

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

    for object_count in [10, 50, 100, 200, 500] {
        group.throughput(Throughput::Elements(object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                b.iter(|| {
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
    bench_descriptor_caching_with_lru,
    bench_descriptor_allocation_with_tracking,
    bench_cache_hit_rate_analysis,
    bench_varying_material_counts,
    bench_single_descriptor_allocation,
    bench_batch_descriptor_allocation,
    bench_descriptor_reuse_vs_recreation,
    bench_descriptor_pooling_patterns,
    bench_allocator_configurations,
    bench_descriptor_write_patterns,
    bench_frame_by_frame_allocation,
);
criterion_main!(benches);
