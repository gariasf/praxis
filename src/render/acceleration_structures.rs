use std::collections::HashSet;

use crate::assets::MeshPool;
use crate::render::bind_groups::{DYNAMIC_TLAS_MARKERS, compute_pipeline, pipeline_layout};
use crate::render::instance::ANCHOR_CELL_SIZE;
use crate::render::readback::PassQueries;
use crate::render::{FrameData, InstanceData, RenderCapacities, shaders};
use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// Moving Instances the dynamic TLAS holds: those in the lowest slots, so
/// which ones changes only as they spawn and despawn; any past it are left
/// out, so they cast no shadow, and are counted and warned about. Provisional
/// until ADR 0013's split of the 8 GB Budget sizes it.
pub const DYNAMIC_TLAS_CAPACITY: u32 = 10_000;

/// The dynamic TLAS's build's name in frame stats: the GPU time from the end
/// of the marker before it to the start of the marker after it.
pub(crate) const DYNAMIC_TLAS_BUILD: &str = "dynamic_tlas_build";

/// NVIDIA's PCI vendor ID (the PCI ID Repository's `pci.ids`
/// [ref:pci-vendor-id-nvidia]), which Vulkan [ref:vulkan-vendorid-pci] and
/// DXGI [ref:dxgi-adapter-desc2-vendorid] both report, the latter through
/// wgpu-hal's DX12 adapter [ref:wgpu-30-dx12-vendor-id].
const NVIDIA_VENDOR: u32 = 0x10DE;

/// The fraction of its built bytes a compacted BLAS keeps on `adapter`, as
/// measured there: an estimate, since wgpu-core keeps the compacted size it
/// reads crate-private [ref:wgpu-30-blas-compacted-size-private], and reading
/// it again takes a query recorded through wgpu-hal outside wgpu's tracking
/// and another wait. NVIDIA's on an NVIDIA GPU, on any backend; lavapipe's on
/// any other Vulkan adapter, software or not; and the M4 Pro's on any other,
/// as the Budgets' reference machine (ADR 0013).
pub fn compacted_blas_fraction(adapter: &wgpu::AdapterInfo) -> f64 {
    if adapter.vendor == NVIDIA_VENDOR {
        // The RTX 2070 SUPER on Vulkan (NVIDIA 621.01), bytes a triangle by
        // wgpu's allocator report (`docs/performance.md`, Measurements
        // behind the code).
        29.2 / 70.7
    } else if adapter.backend == wgpu::Backend::Vulkan {
        // lavapipe (Mesa 25.2.8) leaves a compacted BLAS as it was built
        // (`docs/research/source-audit.md`, lavapipe's figures).
        1.0
    } else {
        // The M4 Pro on Metal, bytes a triangle by `currentAllocatedSize`
        // (`docs/research/global-illumination-and-lighting.md`).
        45.5 / 94.0
    }
}

/// Bytes of a BLAS of `geometry` built with `flags` on `device`: the size its
/// driver gives through wgpu-hal, asked as wgpu-core asks it to create the
/// BLAS. `input` is any buffer of `device`'s with a BLAS or TLAS input usage,
/// which the query names in place of wgpu-core's zero buffer: Metal's
/// requires one [ref:wgpu-hal-30-metal-instances-buffer-unwrap].
pub fn blas_bytes(
    device: &wgpu::Device,
    input: &wgpu::Buffer,
    flags: wgpu::AccelerationStructureFlags,
    geometry: &wgpu::BlasTriangleGeometrySizeDescriptor,
) -> u64 {
    built_bytes(device, input, flags, Geometry::Triangles(geometry))
}

/// Bytes of a TLAS of `descriptor` on `device`: the size its driver gives
/// through wgpu-hal, asked as wgpu-core asks it to create the TLAS, and
/// wgpu-core's buffer of its Instances. `input` is as `blas_bytes` takes it.
pub fn tlas_bytes(
    device: &wgpu::Device,
    input: &wgpu::Buffer,
    descriptor: &wgpu::CreateTlasDescriptor<'_>,
) -> u64 {
    built_bytes(
        device,
        input,
        descriptor.flags,
        Geometry::Instances(descriptor.max_instances),
    )
}

enum Geometry<'a> {
    Triangles(&'a wgpu::BlasTriangleGeometrySizeDescriptor),
    Instances(u32),
}

fn built_bytes(
    device: &wgpu::Device,
    input: &wgpu::Buffer,
    flags: wgpu::AccelerationStructureFlags,
    geometry: Geometry<'_>,
) -> u64 {
    let bytes: Option<u64> = None;
    #[cfg(target_vendor = "apple")]
    let bytes =
        bytes.or_else(|| hal_built_bytes::<wgpu::hal::api::Metal>(device, input, flags, &geometry));
    #[cfg(any(
        windows,
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd"
    ))]
    let bytes = bytes
        .or_else(|| hal_built_bytes::<wgpu::hal::api::Vulkan>(device, input, flags, &geometry));
    #[cfg(windows)]
    let bytes =
        bytes.or_else(|| hal_built_bytes::<wgpu::hal::api::Dx12>(device, input, flags, &geometry));
    bytes.unwrap_or_else(|| {
        tracing::error!("Acceleration structure counted as 0 bytes: its backend has no size query");
        0
    })
}

/// `built_bytes` on backend `A`, or None when `device` is not `A`'s, through
/// wgpu-hal's size query [ref:wgpu-30-device-as-hal] [ref:wgpu-hal-30-build-sizes].
/// Each geometry is as wgpu-core gives it to the same query
/// [ref:wgpu-30-blas-size-at-create] [ref:wgpu-30-tlas-size-at-create], and
/// the Instance buffer a slot of wgpu-hal's Instance encoding each, for at
/// least one slot [ref:wgpu-30-tlas-instance-buffer]
/// [ref:wgpu-hal-30-metal-instance-bytes] [ref:wgpu-hal-30-instance-bytes].
fn hal_built_bytes<A: wgpu::hal::Api>(
    device: &wgpu::Device,
    input: &wgpu::Buffer,
    flags: wgpu::AccelerationStructureFlags,
    geometry: &Geometry<'_>,
) -> Option<u64> {
    use wgpu::hal::{self, Device as _};
    // SAFETY: neither object is destroyed through its handle.
    let (hal_device, hal_buffer) = unsafe { (device.as_hal::<A>()?, input.as_hal::<A>()?) };
    let buffer = Some(&*hal_buffer);
    let entries = match *geometry {
        Geometry::Triangles(triangles) => hal::AccelerationStructureEntries::Triangles(vec![
            hal::AccelerationStructureTriangles {
                vertex_buffer: buffer,
                vertex_format: triangles.vertex_format,
                first_vertex: 0,
                vertex_count: triangles.vertex_count,
                vertex_stride: 0,
                indices: triangles.index_format.zip(triangles.index_count).map(
                    |(format, count)| hal::AccelerationStructureTriangleIndices {
                        format,
                        buffer,
                        offset: 0,
                        count,
                    },
                ),
                transform: flags
                    .contains(wgpu::AccelerationStructureFlags::USE_TRANSFORM)
                    .then(|| hal::AccelerationStructureTriangleTransform {
                        buffer: &*hal_buffer,
                        offset: 0,
                    }),
                flags: triangles.flags,
            },
        ]),
        Geometry::Instances(count) => {
            hal::AccelerationStructureEntries::Instances(hal::AccelerationStructureInstances {
                buffer,
                offset: 0,
                count,
            })
        }
    };
    // SAFETY: the descriptor names live buffers only, as wgpu-core's does.
    let sizes = unsafe {
        hal_device.get_acceleration_structure_build_sizes(
            &hal::GetAccelerationStructureBuildSizesDescriptor {
                entries: &entries,
                flags,
            },
        )
    };
    let instance_buffer_bytes = match *geometry {
        Geometry::Triangles(_) => 0,
        Geometry::Instances(count) => {
            let instance = hal_device.tlas_instance_to_bytes(hal::TlasInstance {
                transform: [0.0; 12],
                custom_data: 0,
                mask: 0,
                blas_address: 0,
                pipeline_intersection_data_offset: 0,
            });
            instance.len() as u64 * u64::from(count.max(1))
        }
    };
    Some(sizes.acceleration_structure_size + instance_buffer_bytes)
}

/// The static TLAS (ADR 0020): each static Instance in its slot's entry,
/// tagged with its slot, in a frame whose origin is the corner of the
/// camera's Anchor in the first Frame data it takes, never re-anchored in M3.
/// It is built from that Frame data, empty or
/// not, so rays can always query it, and rebuilt in a frame that changes a
/// static Instance or frees one's slot.
pub struct StaticTlas {
    tlas: Counted<wgpu::Tlas>,
    anchor: Option<[i32; 3]>,
    /// Slots past its capacity that hold a static Instance.
    left_out: HashSet<u32>,
}

impl StaticTlas {
    /// An empty static TLAS of `capacity` slots, counted in `memory`.
    pub fn new(device: &wgpu::Device, memory: &GpuMemory, capacity: u32) -> Self {
        Self {
            tlas: memory.create_tlas(
                device,
                &wgpu::CreateTlasDescriptor {
                    label: Some("Static TLAS"),
                    max_instances: capacity,
                    // Rebuilt only when its Instances change, traced every frame.
                    flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                    update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                },
            ),
            anchor: None,
            left_out: HashSet::new(),
        }
    }

    /// Takes `frame`'s Instance writes, each static one's BLAS from
    /// `mesh_pool`, and records its build into `encoder` for the first Frame
    /// data and when they change a static Instance or free one's slot.
    /// Returns whether it did.
    pub fn update(
        &mut self,
        frame: &FrameData,
        mesh_pool: &MeshPool,
        encoder: &mut wgpu::CommandEncoder,
    ) -> bool {
        let mut changed = self.anchor.is_none();
        let anchor = *self.anchor.get_or_insert(frame.camera.anchor);
        let was_leaving_out = !self.left_out.is_empty();
        for write in &frame.instance_writes {
            let instances = &frame.instances[write.instances.clone()];
            for (slot, instance) in (write.first_slot..).zip(instances) {
                // A freed slot's Instance has no Mesh, so no BLAS.
                let blas = (!write.moving)
                    .then(|| {
                        mesh_pool
                            .handle_at(instance.mesh)
                            .and_then(|handle| mesh_pool.blas(handle))
                    })
                    .flatten();
                let Some(entry) = self.tlas.get_mut_single(slot as usize) else {
                    if blas.is_some() {
                        self.left_out.insert(slot);
                    } else {
                        self.left_out.remove(&slot);
                    }
                    continue;
                };
                *entry = match blas {
                    Some(blas) => Some(tlas_instance(
                        blas,
                        anchored_transform(instance, anchor),
                        slot,
                    )),
                    None if entry.is_none() => continue,
                    None => None,
                };
                changed = true;
            }
        }
        if !was_leaving_out && !self.left_out.is_empty() {
            tracing::warn!(
                left_out = self.left_out.len(),
                capacity = self.tlas.get().len(),
                "Static Instances left out of the static TLAS: their slots are past its capacity, so they cast no shadow"
            );
        }
        if changed {
            encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&*self.tlas));
        }
        changed
    }

    /// Its binding, which stays valid as it is rebuilt in place.
    pub fn binding(&self) -> wgpu::BindingResource<'_> {
        self.tlas.as_binding()
    }

    /// The Anchor whose corner is its frame's origin: the camera's in the
    /// first Frame data it took, zero before.
    pub fn anchor(&self) -> [i32; 3] {
        self.anchor.unwrap_or_default()
    }

    /// Static Instances in slots past its capacity, which it leaves out.
    pub fn instances_left_out(&self) -> u32 {
        self.left_out.len() as u32
    }
}

/// The dynamic TLAS (ADR 0020): the Moving Instances that Frame data lists,
/// each tagged with its slot, rebuilt each frame in the static TLAS's frame,
/// so a ray composed once is traced against both. Its entries are filled
/// before the frame's uploads, which carry the box that holds them, and its
/// build goes in a command buffer of its own, so its recording and its
/// submit are timed on their own, between two markers that time it on the
/// GPU.
pub struct DynamicTlas {
    tlas: Counted<wgpu::Tlas>,
    /// Entries its last fill filled, from the first.
    filled: usize,
    /// The bytes the build of its last fill uploads, until `record` records
    /// it; none when that fill left nothing to build.
    unrecorded_upload_bytes: Option<u64>,
    /// Bytes each entry uploads to the GPU per build: the backend's own
    /// instance record, which wgpu-core converts it to and stages
    /// [ref:wgpu-30-tlas-instance-staging].
    entry_upload_bytes: u64,
    /// One invocation that queries the TLAS, recorded on either side of its
    /// build: the GPU runs neither marker alongside the build, as it would a
    /// pass that does not touch the TLAS, and a pass that does nothing writes
    /// no timestamp on Metal.
    marker_pipeline: wgpu::ComputePipeline,
    /// Its group 3 (`DYNAMIC_TLAS_MARKERS`).
    marker_bind_group: wgpu::BindGroup,
    /// What the markers found, which nothing reads.
    _marker_result: Counted<wgpu::Buffer>,
    /// The encoder `prepare_encoder` created for the next `record`.
    encoder: Option<wgpu::CommandEncoder>,
}

impl DynamicTlas {
    /// An empty dynamic TLAS of `DYNAMIC_TLAS_CAPACITY` entries on
    /// `backend`, built through `queue` at once, so a marker never queries it
    /// unbuilt, its markers' shader module assembled with `capacities`.
    pub fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        queue: &wgpu::Queue,
        backend: wgpu::Backend,
        capacities: &RenderCapacities,
    ) -> Self {
        let tlas = memory.create_tlas(
            device,
            &wgpu::CreateTlasDescriptor {
                label: Some("Dynamic TLAS"),
                max_instances: DYNAMIC_TLAS_CAPACITY,
                // Traced at every lit pixel, as the static TLAS is; ADR 0020's
                // per-frame cost was measured with this flag.
                flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            },
        );
        let mut encoder = create_encoder(device);
        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&*tlas));
        queue.submit(std::iter::once(encoder.finish()));
        let marker_result = memory.create_buffer(
            device,
            GpuMemoryKind::SceneAndPassBuffers,
            &wgpu::BufferDescriptor {
                label: Some("Dynamic TLAS Marker Result"),
                size: size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            },
        );
        let layout = DYNAMIC_TLAS_MARKERS.layout(device);
        let marker_bind_group = DYNAMIC_TLAS_MARKERS.bind_group(
            device,
            &layout,
            [
                ("dynamic_tlas", tlas.as_binding()),
                ("marker_result", marker_result.as_entire_binding()),
            ],
        );
        let shader =
            device.create_shader_module(shaders::DYNAMIC_TLAS_MARKER.descriptor(capacities));
        let groups = [(&DYNAMIC_TLAS_MARKERS, &layout)];
        let label = "Dynamic TLAS Marker Pipeline Layout";
        let pipeline_layout = pipeline_layout(device, label, &groups);
        let marker_pipeline =
            compute_pipeline(device, &pipeline_layout, &shader, "mark_dynamic_tlas", &[]);
        Self {
            tlas,
            filled: 0,
            unrecorded_upload_bytes: None,
            entry_upload_bytes: match backend {
                // `MTLIndirectAccelerationStructureInstanceDescriptor`
                // [ref:metal-indirect-instance-descriptor].
                wgpu::Backend::Metal => 72,
                // `VkAccelerationStructureInstanceKHR`
                // [ref:vulkan-acceleration-structure-instance],
                // `D3D12_RAYTRACING_INSTANCE_DESC`
                // [ref:d3d12-raytracing-instance-desc].
                _ => 64,
            },
            marker_pipeline,
            marker_bind_group,
            _marker_result: marker_result,
            encoder: None,
        }
    }

    /// Fills its entries with `frame`'s Moving Instances, each with its
    /// BLAS from `mesh_pool` and its transform from the corner of `anchor`,
    /// the static TLAS's, for `record` to build. Returns a box in that frame
    /// that holds every triangle they place, its lowest corner then its
    /// highest: each one's Mesh bounds, which hold every shape a bend gives
    /// it, placed by its transform; a point at the origin when they place
    /// none.
    pub fn fill(
        &mut self,
        frame: &FrameData,
        mesh_pool: &MeshPool,
        anchor: [i32; 3],
    ) -> [[f32; 3]; 2] {
        // Prepare lists no more than its capacity.
        let listed = frame
            .moving_instances
            .iter()
            .take(DYNAMIC_TLAS_CAPACITY as usize);
        let filled = listed.len();
        let previously_filled = std::mem::replace(&mut self.filled, filled);
        let mut uploaded_entries = 0;
        let mut bounds: Option<[glam::Vec3; 2]> = None;
        for (index, moving) in listed.enumerate() {
            let instance = &frame.instances[moving.instance];
            let mesh = mesh_pool
                .handle_at(instance.mesh)
                .and_then(|handle| mesh_pool.blas(handle).zip(mesh_pool.get(handle)));
            self.tlas[index] = mesh.map(|(blas, mesh)| {
                let transform = anchored_transform(instance, anchor);
                let [lowest, highest] = placed_bounds(&transform, mesh.bounds);
                bounds = Some(
                    bounds.map_or([lowest, highest], |[held_lowest, held_highest]| {
                        [held_lowest.min(lowest), held_highest.max(highest)]
                    }),
                );
                uploaded_entries += 1;
                tlas_instance(blas, transform, moving.slot)
            });
        }
        for index in filled..previously_filled {
            self.tlas[index] = None;
        }
        self.unrecorded_upload_bytes = (filled > 0 || previously_filled > 0)
            .then_some(uploaded_entries * self.entry_upload_bytes);
        bounds.unwrap_or_default().map(<[f32; 3]>::from)
    }

    /// Records the build of its last fill into a command buffer of
    /// `device`'s, between markers `queries` times, the end of the
    /// first and the start of the second. Returns the command buffer and the
    /// bytes the build uploads; `None` when that fill left nothing to build,
    /// as it held none and held none in the fill before, or was recorded.
    pub fn record(
        &mut self,
        device: &wgpu::Device,
        queries: Option<PassQueries<'_, '_>>,
    ) -> Option<(wgpu::CommandBuffer, u64)> {
        let uploaded_bytes = self.unrecorded_upload_bytes.take()?;
        let mut encoder = self
            .encoder
            .take()
            .unwrap_or_else(|| create_encoder(device));
        let [before, after] = queries.map_or([None, None], |queries| queries.bracket().map(Some));
        self.mark(&mut encoder, before);
        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&*self.tlas));
        self.mark(&mut encoder, after);
        Some((encoder.finish(), uploaded_bytes))
    }

    /// Creates the encoder its next `record` records into, unless it holds
    /// one: called before the frame creates its own. A new encoder reuses
    /// the encoder the GPU finished with last [ref:wgpu-30-command-encoder-pool],
    /// and a frame submits the dynamic TLAS's build before its passes, so the
    /// build's encoder is released first. Created in this order, when no
    /// other submission takes an encoder, the two swap roles every frame, so
    /// every encoder the frames reuse records the frame's passes within its
    /// first two frames, and the state Metal keeps per encoder, which grows
    /// to the largest pipeline and bindings it recorded and never shrinks
    /// [ref:wgpu-hal-30-metal-command-state], has grown by then. Created in
    /// the other order, each keeps its role, until the GPU finishes a frame's
    /// build but not its passes before the next frame starts: that frame's
    /// passes then reuse the build's encoder, whose state grows in a frame
    /// the GPU's timing picks. An encoder held over a frame that builds
    /// nothing breaks the swap for a frame, harmlessly.
    pub fn prepare_encoder(&mut self, device: &wgpu::Device) {
        if self.encoder.is_none() {
            self.encoder = Some(create_encoder(device));
        }
    }

    /// Records a marker into `encoder`, timed into `timestamp_writes`; none
    /// without them. wgpu-core checks the first marker's query against the
    /// BLASes of the last build, so a BLAS rebuilt through wgpu since would
    /// fail the whole command buffer, build included.
    fn mark(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        if timestamp_writes.is_none() {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Dynamic TLAS Marker"),
            timestamp_writes,
        });
        pass.set_pipeline(&self.marker_pipeline);
        pass.set_bind_group(DYNAMIC_TLAS_MARKERS.index, &self.marker_bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }

    /// Its binding, which stays valid as it is rebuilt in place.
    pub fn binding(&self) -> wgpu::BindingResource<'_> {
        self.tlas.as_binding()
    }
}

/// An Instance's entry in a TLAS: `blas`, placed by `transform`
/// (`anchored_transform`) and tagged with its `slot`.
fn tlas_instance(blas: &wgpu::Blas, transform: [f32; 12], slot: u32) -> wgpu::TlasInstance {
    // Every ray's cull mask.
    wgpu::TlasInstance::new(blas, transform, slot, 0xff)
}

/// The share of a Mesh's largest coordinate its box grows by before it is
/// placed, for the bend pass's rounding: under Metal's fast math, sine errs
/// by up to 2^-13 within ±π [ref:msl-4.1-sin-fast-accuracy], and a bent
/// vertex, built from a few sines and cosines over its height and its
/// distance from the middle, lands off its exact shape by a few times that
/// share of its distance from the Mesh's origin, by estimate, as no
/// specification bounds cosine or angles past ±π, which an azimuth can be.
/// Every Mesh grows, as which ones bend is not known here.
const BEND_ROUNDING_SHARE: f32 = 0.00390625; // 2^-8

fn create_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Dynamic TLAS Encoder"),
    })
}

/// The box that holds `bounds`, a Mesh's box, its lowest corner then its
/// highest, grown by `BEND_ROUNDING_SHARE` and placed by `transform`, a TLAS
/// entry's rows: the box's centre placed, and each row's absolute values
/// over its half-extent either side.
fn placed_bounds(transform: &[f32; 12], bounds: [[f32; 3]; 2]) -> [glam::Vec3; 2] {
    let [lowest, highest] = bounds.map(glam::Vec3::from);
    let centre = 0.5 * (lowest + highest);
    let largest_coordinate = lowest.abs().max(highest.abs()).max_element();
    let half_extent = 0.5 * (highest - lowest) + BEND_ROUNDING_SHARE * largest_coordinate;
    let rows: [[f32; 4]; 3] = bytemuck::cast(*transform);
    let row = |axis: usize| glam::Vec4::from(rows[axis]).truncate();
    let placed_centre =
        glam::Vec3::from_array([0, 1, 2].map(|axis| row(axis).dot(centre) + rows[axis][3]));
    let reach = glam::Vec3::from_array([0, 1, 2].map(|axis| row(axis).abs().dot(half_extent)));
    [placed_centre - reach, placed_centre + reach]
}

/// `instance`'s transform as rows of the 3×4 matrix a TLAS Instance holds,
/// its translation from `anchor`'s corner composed in f64, so only the f32
/// result rounds (ADR 0014).
fn anchored_transform(instance: &InstanceData, anchor: [i32; 3]) -> [f32; 12] {
    let [x_axis, y_axis, z_axis] = instance.rotation_scale;
    let row = |axis: usize| {
        let cells = i64::from(instance.anchor[axis]) - i64::from(anchor[axis]);
        let translation = cells as f64 * ANCHOR_CELL_SIZE + f64::from(instance.offset[axis]);
        [x_axis[axis], y_axis[axis], z_axis[axis], translation as f32]
    };
    bytemuck::cast([0, 1, 2].map(row))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::assets::pools::tests::{logged, mesh, trace_down, triangle_about_x};
    use crate::assets::{MeshHandle, PoolCapacities};
    use crate::gpu::GpuContext;
    use crate::render::instance::padded_columns;
    use crate::render::prepare::{InstanceWrite, MovingInstance};
    use crate::render::{CameraUniform, LoadStaging, NO_MATERIAL, NO_MESH};
    use crate::stats::gpu_memory::tests::gpu_memory;

    /// The camera's Anchor in the first frame: 100 km out, where an
    /// Instance's World position would round in f32 and its offset from the
    /// anchor does not.
    const LOAD_ANCHOR: [i32; 3] = [1562, 0, -3];

    /// An NVIDIA GPU's compacted BLASes are counted at NVIDIA's fraction on
    /// Vulkan and on DX12, lavapipe's, a CPU Vulkan adapter under Mesa's
    /// vendor ID (the Vulkan registry's `VK_VENDOR_ID_MESA`
    /// [ref:vulkan-vendor-id-mesa], which lavapipe reports
    /// [ref:lavapipe-vendor-id-mesa]), at lavapipe's, and an Apple GPU's on
    /// Metal at the M4 Pro's.
    #[test]
    fn the_compacted_blas_fraction_follows_the_adapter() {
        let adapter = |device_type, backend, vendor| wgpu::AdapterInfo {
            vendor,
            ..wgpu::AdapterInfo::new(device_type, backend)
        };
        let discrete = wgpu::DeviceType::DiscreteGpu;
        for (adapter, expected) in [
            (
                adapter(discrete, wgpu::Backend::Vulkan, 0x10DE),
                29.2 / 70.7,
            ),
            (adapter(discrete, wgpu::Backend::Dx12, 0x10DE), 29.2 / 70.7),
            (
                adapter(wgpu::DeviceType::Cpu, wgpu::Backend::Vulkan, 0x10005),
                1.0,
            ),
            (
                adapter(wgpu::DeviceType::IntegratedGpu, wgpu::Backend::Metal, 0),
                45.5 / 94.0,
            ),
        ] {
            assert_eq!(compacted_blas_fraction(&adapter), expected, "{adapter:?}");
        }
    }

    /// A Mesh pool holding `triangle_about_x`'s Mesh after another, its BLAS
    /// built and compacted.
    struct Triangle {
        context: GpuContext,
        mesh_pool: MeshPool,
        mesh: MeshHandle,
    }

    impl Triangle {
        /// None, after saying so, when there is no GPU.
        fn new() -> Option<Self> {
            let context = match pollster::block_on(GpuContext::headless()) {
                Ok(context) => context,
                Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                    // Not `eprintln!`, which libtest captures from passing tests.
                    writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                    return None;
                }
                Err(error) => panic!("headless GPU context failed: {error:#}"),
            };
            let mut staging = LoadStaging::new(context.device.clone());
            let mut mesh_pool = MeshPool::new(
                &context.device,
                &gpu_memory(&context),
                &PoolCapacities::checked_in(),
            );
            let mut load = |mesh| mesh_pool.load(&mut staging, &mesh, &[None]);
            load(mesh(3, vec![0, 1, 2])).expect("a triangle fits");
            let mesh = load(triangle_about_x()).expect("a triangle fits");
            staging.submit(&context.queue);
            mesh_pool.compact_blases(&context.queue);
            Some(Self {
                context,
                mesh_pool,
                mesh,
            })
        }

        /// An Instance of the triangle with `rotation_scale`, its pivot at
        /// `offset` from the corner of `anchor`.
        fn instance(
            &self,
            anchor: [i32; 3],
            offset: [f32; 3],
            rotation_scale: glam::Mat3,
        ) -> InstanceData {
            InstanceData {
                rotation_scale: padded_columns(rotation_scale),
                normal_matrix: padded_columns(glam::Mat3::IDENTITY),
                anchor,
                material_id: NO_MATERIAL,
                offset,
                mesh: self.mesh.index,
            }
        }

        /// Updates `tlas` with Frame data that writes each of `writes`, a
        /// slot, its Instance and whether it is a Moving Instance, seen from
        /// a camera in `camera_anchor`; returns whether it rebuilt.
        fn update(
            &self,
            tlas: &mut StaticTlas,
            camera_anchor: [i32; 3],
            writes: Vec<(u32, InstanceData, bool)>,
        ) -> bool {
            let mut frame = FrameData {
                camera: CameraUniform {
                    anchor: camera_anchor,
                    ..CameraUniform::default()
                },
                ..FrameData::default()
            };
            for (slot, instance, moving) in writes {
                let index = frame.instances.len();
                frame.instances.push(instance);
                frame.instance_writes.push(InstanceWrite {
                    first_slot: slot,
                    instances: index..index + 1,
                    moving,
                });
            }
            let device = &self.context.device;
            let mut encoder = device.create_command_encoder(&Default::default());
            let rebuilt = tlas.update(&frame, &self.mesh_pool, &mut encoder);
            self.context.queue.submit(std::iter::once(encoder.finish()));
            rebuilt
        }
    }

    /// Each entry the static TLAS holds: its slot, the slot it is tagged
    /// with, and its transform.
    fn entries(tlas: &StaticTlas) -> Vec<(usize, u32, [f32; 12])> {
        let entries = tlas.tlas.get().iter().enumerate();
        entries
            .filter_map(|(slot, entry)| {
                let instance = entry.as_ref()?;
                Some((slot, instance.custom_data, instance.transform))
            })
            .collect()
    }

    /// The Instance a freed slot is written.
    fn freed() -> InstanceData {
        InstanceData {
            mesh: NO_MESH,
            ..bytemuck::Zeroable::zeroed()
        }
    }

    /// An entry's custom data is its slot, of which wgpu keeps the lower 24
    /// bits [ref:wgpu-30-tlas-instance-fields].
    #[test]
    fn every_slot_the_checked_in_budgets_give_fits_an_entrys_custom_data() {
        let capacities = RenderCapacities::checked_in();
        assert!(capacities.instances <= 1 << 24);
        assert!(capacities.static_tlas_instances <= 1 << 24);
    }

    #[test]
    fn the_static_tlas_holds_each_static_instance_in_its_slot_from_the_load_anchor() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        // The first Frame data builds it, even with only Moving Instances.
        let memory = gpu_memory(&triangle.context);
        let mut empty = StaticTlas::new(&triangle.context.device, &memory, 8);
        let moving = triangle.instance(LOAD_ANCHOR, [0.0; 3], glam::Mat3::IDENTITY);
        assert!(triangle.update(&mut empty, LOAD_ANCHOR, vec![(0, moving, true)]));
        assert_eq!(
            trace_down(&triangle.context, &empty.tlas, &[[0.8, 0.0, 0.0]]),
            [None]
        );
        assert!(!triangle.update(&mut empty, LOAD_ANCHOR, vec![(0, moving, true)]));

        let mut tlas = StaticTlas::new(&triangle.context.device, &memory, 8);
        // A quarter turn about +Z, then a scale by 2: +X goes to +2Y.
        let turned_and_doubled = glam::Mat3::from_cols(
            glam::vec3(0.0, 2.0, 0.0),
            glam::vec3(-2.0, 0.0, 0.0),
            glam::vec3(0.0, 0.0, 2.0),
        );
        let turned = triangle.instance([1563, -1, -3], [0.25, 63.5, 10.0], turned_and_doubled);
        let unturned = |x| triangle.instance(LOAD_ANCHOR, [x, 0.0, 0.0], glam::Mat3::IDENTITY);
        let writes = vec![
            (0, turned, false),
            (1, unturned(0.0), true),
            (2, unturned(5.0), false),
        ];
        assert!(triangle.update(&mut tlas, LOAD_ANCHOR, writes));
        // Rows of the 3×4 matrix, the translation from the load Anchor's
        // corner: one cell along +X, one down, plus the offset.
        let expected = vec![
            (
                0,
                0,
                [
                    0.0, -2.0, 0.0, 64.25, 2.0, 0.0, 0.0, -0.5, 0.0, 0.0, 2.0, 10.0,
                ],
            ),
            (
                2,
                2,
                [1.0, 0.0, 0.0, 5.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            ),
        ];
        assert_eq!(entries(&tlas), expected);
        // On the GPU, the turned triangle lands at its pivot plus 2 × (0, 0.8),
        // where a transposed rotation would put it at (0, -1.6); the Moving
        // Instance's is not there.
        let points = [
            [64.25, 1.1, 10.0],
            [64.25, -2.1, 10.0],
            [0.8, 0.0, 0.0],
            [5.8, 0.0, 0.0],
        ];
        let traced = trace_down(&triangle.context, &tlas.tlas, &points);
        assert_eq!(traced, [Some(0), None, None, Some(2)]);

        // The camera moves on and only the Moving Instance is written: no
        // rebuild, and the frame keeps the load Anchor.
        let away = [1600, 2, -3];
        assert!(!triangle.update(&mut tlas, away, vec![(1, unturned(0.0), true)]));
        // A static spawn and a despawn each rebuild it in their frame.
        assert!(triangle.update(&mut tlas, away, vec![(3, unturned(-4.0), false)]));
        assert!(triangle.update(&mut tlas, away, vec![(2, freed(), false)]));
        let traced = trace_down(
            &triangle.context,
            &tlas.tlas,
            &[[-3.2, 0.0, 0.0], points[3]],
        );
        assert_eq!(traced, [Some(3), None]);
        let slots: Vec<usize> = entries(&tlas).iter().map(|&(slot, ..)| slot).collect();
        assert_eq!(slots, [0, 3]);
        // Freeing a Moving Instance's slot leaves it as it was.
        assert!(!triangle.update(&mut tlas, away, vec![(1, freed(), false)]));
    }

    /// The dynamic TLAS's fill returns the box that holds each Moving
    /// Instance's Mesh bounds, grown for the bend pass's rounding, where its
    /// transform from the static TLAS's Anchor places them, and a point at the origin once it holds none. A
    /// fill leaves one build to record, but an empty fill after an empty one.
    #[test]
    fn the_dynamic_tlas_returns_the_box_that_holds_its_moving_instances_bounds() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        let context = &triangle.context;
        let mut tlas = DynamicTlas::new(
            &context.device,
            &gpu_memory(context),
            &context.queue,
            context.backend(),
            &RenderCapacities::checked_in(),
        );
        // A quarter turn about +Z, then a scale by 2: +X goes to +2Y.
        let turned_and_doubled = glam::Mat3::from_cols(
            glam::vec3(0.0, 2.0, 0.0),
            glam::vec3(-2.0, 0.0, 0.0),
            glam::vec3(0.0, 0.0, 2.0),
        );
        let frame = FrameData {
            instances: vec![
                triangle.instance([1563, -1, -3], [0.25, 63.5, 10.0], turned_and_doubled),
                triangle.instance(LOAD_ANCHOR, [-4.0, 0.0, 0.0], glam::Mat3::IDENTITY),
            ],
            moving_instances: vec![
                MovingInstance {
                    slot: 0,
                    instance: 0,
                },
                MovingInstance {
                    slot: 7,
                    instance: 1,
                },
            ],
            ..FrameData::default()
        };
        let [lowest, highest] = tlas.fill(&frame, &triangle.mesh_pool, LOAD_ANCHOR);
        // The triangle spans 0.6 to 1.0 in X and ±0.2 in Y, and its box
        // grows by 2^-8 of 1.0 m on every side. The turned one is doubled
        // and turned about its pivot at (64.25, -0.5, 10.0); the unturned
        // one sits 4 m below the Anchor's corner in X.
        let grown = BEND_ROUNDING_SHARE;
        let expected = [
            [-3.4 - grown, -0.2 - grown, -grown],
            [64.65 + 2.0 * grown, 1.5 + 2.0 * grown, 10.0 + 2.0 * grown],
        ];
        for (actual, expected) in [lowest, highest].iter().zip(expected) {
            let distance = glam::Vec3::from(*actual).distance(glam::Vec3::from(expected));
            assert!(distance < 1e-5, "{actual:?}, expected {expected:?}");
        }
        let built =
            |tlas: &mut DynamicTlas| tlas.record(&context.device, None).map(|(_, bytes)| bytes);
        assert_eq!(built(&mut tlas), Some(2 * tlas.entry_upload_bytes));
        assert_eq!(built(&mut tlas), None);

        let empty = FrameData::default();
        assert_eq!(
            tlas.fill(&empty, &triangle.mesh_pool, LOAD_ANCHOR),
            [[0.0; 3]; 2]
        );
        assert_eq!(built(&mut tlas), Some(0));
        tlas.fill(&empty, &triangle.mesh_pool, LOAD_ANCHOR);
        assert_eq!(built(&mut tlas), None);
    }

    #[test]
    fn static_instances_past_its_capacity_are_left_out_counted_and_warned_about_once() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        let mut tlas = StaticTlas::new(&triangle.context.device, &gpu_memory(&triangle.context), 2);
        let instance = triangle.instance(LOAD_ANCHOR, [0.0; 3], glam::Mat3::IDENTITY);
        let ((), log) = logged(|| {
            let writes = (0..4).map(|slot| (slot, instance, false)).collect();
            triangle.update(&mut tlas, LOAD_ANCHOR, writes);
            assert_eq!(tlas.instances_left_out(), 2);
            triangle.update(&mut tlas, LOAD_ANCHOR, vec![(3, freed(), false)]);
            assert_eq!(tlas.instances_left_out(), 1);
            triangle.update(&mut tlas, LOAD_ANCHOR, vec![(2, instance, false)]);
            assert_eq!(tlas.instances_left_out(), 1);
        });
        assert_eq!(entries(&tlas).len(), 2);
        let warnings = log.matches("left out of the static TLAS").count();
        assert_eq!(warnings, 1, "{log}");
    }
}
