// Throwaway probe, never merged: what acceleration structures allocate on an
// adapter with an allocator report (lavapipe), by allocation name.
#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::gpu::GpuContext;

    fn allocated(device: &wgpu::Device, prefix: &str) -> Option<u64> {
        let report = device.generate_allocator_report()?;
        Some(
            report
                .allocations
                .iter()
                .filter(|allocation| allocation.name.starts_with(prefix))
                .map(|allocation| allocation.size)
                .sum(),
        )
    }

    fn total(device: &wgpu::Device) -> Option<u64> {
        Some(device.generate_allocator_report()?.total_allocated_bytes)
    }

    #[test]
    fn acceleration_structure_sizes_probe() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) => {
                writeln!(std::io::stderr(), "AS-PROBE no context: {error}").ok();
                return;
            }
        };
        let device = &context.device;
        let queue = &context.queue;
        let info = context.adapter.get_info();
        if device.generate_allocator_report().is_none() {
            writeln!(std::io::stderr(), "AS-PROBE {} has no allocator report", info.name).ok();
            return;
        }
        writeln!(std::io::stderr(), "AS-PROBE adapter {} {:?} driver {} {}", info.name, info.backend, info.driver, info.driver_info).ok();
        let wait = || device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        for quads_per_side in [1u32, 32, 128, 256] {
            // A grid of quads_per_side^2 quads, two triangles each, in 28-byte vertices.
            let side = quads_per_side + 1;
            let mut vertices = Vec::new();
            for z in 0..side {
                for x in 0..side {
                    let height = ((x * 7 + z * 13) % 5) as f32 * 0.1;
                    let words = [x as f32, height, z as f32, 0.0, 0.0, 0.0, 0.0];
                    vertices.extend_from_slice(&words);
                }
            }
            let mut indices = Vec::new();
            for z in 0..quads_per_side {
                for x in 0..quads_per_side {
                    let corner = z * side + x;
                    indices.extend_from_slice(&[corner, corner + side, corner + 1, corner + 1, corner + side, corner + side + 1]);
                }
            }
            let triangles = indices.len() as u64 / 3;
            let buffer = |label, bytes: &[u8]| {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::BLAS_INPUT | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buffer, 0, bytes);
                buffer
            };
            let vertex_buffer = buffer("probe vertices", bytemuck::cast_slice(&vertices));
            let index_buffer = buffer("probe indices", bytemuck::cast_slice(&indices));
            let size = wgpu::BlasTriangleGeometrySizeDescriptor {
                vertex_format: wgpu::VertexFormat::Float32x3,
                vertex_count: side * side,
                index_format: Some(wgpu::IndexFormat::Uint32),
                index_count: Some(indices.len() as u32),
                flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
            };
            let before = total(device).unwrap();
            let blas = device.create_blas(
                &wgpu::CreateBlasDescriptor {
                    label: Some("Probe BLAS"),
                    flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
                        | wgpu::AccelerationStructureFlags::ALLOW_COMPACTION,
                    update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                },
                wgpu::BlasGeometrySizeDescriptors::Triangles {
                    descriptors: vec![size.clone()],
                },
            );
            let created = total(device).unwrap() - before;
            let built_blas_bytes = allocated(device, "Probe BLAS").unwrap();
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.build_acceleration_structures(
                [&wgpu::BlasBuildEntry {
                    blas: &blas,
                    geometry: wgpu::BlasGeometries::TriangleGeometries(vec![
                        wgpu::BlasTriangleGeometry {
                            size: &size,
                            vertex_buffer: &vertex_buffer,
                            first_vertex: 0,
                            vertex_stride: 28,
                            index_buffer: Some(&index_buffer),
                            first_index: Some(0),
                            transform_buffer: None,
                            transform_buffer_offset: None,
                        },
                    ]),
                }],
                [],
            );
            queue.submit([encoder.finish()]);
            wait();
            blas.prepare_compaction_async(|result| {
                writeln!(std::io::stderr(), "AS-PROBE compaction callback {result:?}").ok();
            });
            wait();
            let ready = blas.ready_for_compaction();
            let compacted = ready.then(|| queue.compact_blas(&blas));
            queue.submit([]);
            wait();
            let compacted_bytes = allocated(device, "Probe BLAS (compacted)").unwrap();
            writeln!(
                std::io::stderr(),
                "AS-PROBE BLAS triangles={triangles} created_delta={created} blas_alloc={built_blas_bytes} ({:.2} B/tri) ready={ready} compacted_alloc={compacted_bytes} ({:.2} B/tri)",
                (built_blas_bytes - compacted_bytes) as f64 / triangles as f64,
                compacted_bytes as f64 / triangles as f64,
            )
            .ok();
            drop(compacted);
            drop(blas);
            queue.submit([]);
            wait();
        }
        for instances in [1024u32, 100_064, 1 << 18] {
            let before = total(device).unwrap();
            let tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
                label: Some("Probe TLAS"),
                max_instances: instances,
                flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            });
            let delta = total(device).unwrap() - before;
            let named = allocated(device, "Probe TLAS").unwrap();
            writeln!(
                std::io::stderr(),
                "AS-PROBE TLAS max_instances={instances} total_delta={delta} ({:.2} B/instance) tlas_alloc={named} ({:.2} B/instance)",
                delta as f64 / instances as f64,
                named as f64 / instances as f64,
            )
            .ok();
            drop(tlas);
            queue.submit([]);
            wait();
        }
        let report = device.generate_allocator_report().unwrap();
        let mut names: Vec<_> = report.allocations.iter().map(|allocation| (&allocation.name, allocation.size)).collect();
        names.sort();
        writeln!(std::io::stderr(), "AS-PROBE remaining allocations {names:?}").ok();
    }
}
