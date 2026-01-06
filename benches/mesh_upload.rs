use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use praxis_graphics::mesh::MeshData;
use std::sync::Arc;
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    VulkanLibrary,
};

fn create_test_allocator() -> Arc<StandardMemoryAllocator> {
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
        .expect("Failed to enumerate devices").find(|p| p.properties().device_type == PhysicalDeviceType::DiscreteGpu)
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

    Arc::new(StandardMemoryAllocator::new_default(device))
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
    let allocator = create_test_allocator();

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
                        .upload(allocator.clone())
                        .expect("Failed to upload mesh");
                    black_box(_gpu_mesh);
                });
            },
        );
    }

    group.finish();
}

fn bench_mesh_upload_with_textures(c: &mut Criterion) {
    let allocator = create_test_allocator();

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
                        .upload(allocator.clone())
                        .expect("Failed to upload mesh");
                    black_box(_gpu_mesh);
                });
            },
        );
    }

    group.finish();
}

fn bench_primitive_generation_and_upload(c: &mut Criterion) {
    let allocator = create_test_allocator();

    c.bench_function("simple_triangle_generation_and_upload", |b| {
        b.iter(|| {
            let positions = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
            let indices = vec![0, 1, 2];
            let mesh_data = MeshData::new(positions, indices);
            let _gpu_mesh = mesh_data
                .upload(allocator.clone())
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
                .upload(allocator.clone())
                .expect("Failed to upload mesh");
            black_box(_gpu_mesh);
        });
    });
}

criterion_group!(
    benches,
    bench_mesh_upload,
    bench_mesh_upload_with_textures,
    bench_primitive_generation_and_upload
);
criterion_main!(benches);
