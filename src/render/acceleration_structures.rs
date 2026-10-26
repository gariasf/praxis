use std::collections::HashSet;

use crate::assets::{MeshHandle, MeshPool};
use crate::render::bind_groups::{DYNAMIC_TLAS_MARKERS, pipeline_layout};
use crate::render::instance::ANCHOR_CELL_SIZE;
use crate::render::{FrameData, InstanceData, shaders};

/// Slots the static TLAS covers: a static Instance in a higher slot is left
/// out, so it casts no shadow, and is counted and warned about. Sized for
/// the 250k Instances M3 plans `town` with, no larger: in wgpu a TLAS
/// rebuild's CPU time grows with its capacity, filled or not.
/// Provisional until ADR 0013's split of the 8 GB Budget sizes it.
pub const STATIC_TLAS_CAPACITY: u32 = 1 << 18;

/// Moving Instances the dynamic TLAS holds: those in the lowest slots, so
/// which ones changes only as they spawn and despawn; any past it are left
/// out, so they cast no shadow, and are counted and warned about. Provisional
/// until ADR 0013's split of the 8 GB Budget sizes it.
pub const DYNAMIC_TLAS_CAPACITY: u32 = 10_000;

/// The dynamic TLAS's build's name in frame stats: the GPU time from the end
/// of the marker before it to the start of the marker after it.
pub(crate) const DYNAMIC_TLAS_BUILD: &str = "dynamic_tlas_build";

/// What acceleration structures take in GPU memory, as measured on one
/// adapter: estimates, since wgpu exposes no acceleration-structure size,
/// until M4 sets exact ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationStructureBytes {
    pub per_blas_triangle: f64,
    pub per_compacted_blas_triangle: f64,
    /// Per Instance a TLAS can hold, wgpu-core's buffer of them included.
    pub per_tlas_instance: f64,
}

impl AccelerationStructureBytes {
    /// The M4 Pro's on Metal: `currentAllocatedSize` around creating 2.1M
    /// triangles of BLAS and a 100,064-Instance TLAS
    /// (`docs/research/global-illumination-and-lighting.md`).
    const M4_PRO: Self = Self {
        per_blas_triangle: 94.0,
        per_compacted_blas_triangle: 45.5,
        per_tlas_instance: 288.5,
    };

    /// lavapipe's on Vulkan (Mesa 25.2.8): wgpu's allocator report around
    /// building and compacting BLASes of 2,048 to 131,072 triangles, which
    /// compaction leaves as they were, and creating TLASes of 1,024 to
    /// 262,144 Instances.
    const LAVAPIPE: Self = Self {
        per_blas_triangle: 104.0,
        per_compacted_blas_triangle: 104.0,
        per_tlas_instance: 240.0,
    };

    /// The estimate for `backend`: lavapipe's on Vulkan, software or not,
    /// and the M4 Pro's on any other, as the Budgets' reference machine
    /// (ADR 0013).
    pub fn estimate(backend: wgpu::Backend) -> Self {
        match backend {
            wgpu::Backend::Vulkan => Self::LAVAPIPE,
            _ => Self::M4_PRO,
        }
    }

    /// Bytes of BLASes of `compacted_triangles` compacted and
    /// `uncompacted_triangles` not.
    pub fn blases(&self, compacted_triangles: u64, uncompacted_triangles: u64) -> u64 {
        (compacted_triangles as f64 * self.per_compacted_blas_triangle
            + uncompacted_triangles as f64 * self.per_blas_triangle)
            .round() as u64
    }

    /// Bytes of a TLAS that can hold `instances`.
    pub fn tlas(&self, instances: usize) -> u64 {
        (instances as f64 * self.per_tlas_instance).round() as u64
    }
}

/// The static TLAS (ADR 0020): each static Instance in its slot's entry,
/// tagged with its slot, in a frame whose origin is the corner of the
/// camera's Anchor in the first Frame data it takes, never re-anchored in M3.
/// It is built from that Frame data, empty or
/// not, so rays can always query it, and rebuilt in a frame that changes a
/// static Instance or frees one's slot.
pub struct StaticTlas {
    tlas: wgpu::Tlas,
    anchor: Option<[i32; 3]>,
    /// Slots past its capacity that hold a static Instance.
    left_out: HashSet<u32>,
}

impl StaticTlas {
    /// An empty static TLAS of `capacity` slots.
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        Self {
            tlas: device.create_tlas(&wgpu::CreateTlasDescriptor {
                label: Some("Static TLAS"),
                max_instances: capacity,
                // Rebuilt only when its Instances change, traced every frame.
                flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            }),
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
                    .then(|| mesh_pool.blas(MeshHandle(instance.mesh)))
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
                    Some(blas) => Some(tlas_instance(blas, instance, anchor, slot)),
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
            encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&self.tlas));
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

    /// Its bytes, as `acceleration_structure_bytes` estimates them for its
    /// capacity, whatever it holds.
    pub fn estimated_bytes(
        &self,
        acceleration_structure_bytes: &AccelerationStructureBytes,
    ) -> u64 {
        acceleration_structure_bytes.tlas(self.tlas.get().len())
    }
}

/// The dynamic TLAS (ADR 0020): the Moving Instances that Frame data lists,
/// each tagged with its slot, rebuilt each frame in the static TLAS's frame,
/// so a ray composed once is traced against both. Its build goes in a
/// command buffer of its own, so its recording and its submit are timed on
/// their own, between two markers that time it on the GPU.
pub struct DynamicTlas {
    tlas: wgpu::Tlas,
    /// Entries its last build filled, from the first.
    filled: usize,
    /// Bytes each entry uploads to the GPU per build: the backend's own
    /// instance record, which wgpu-core converts it to and stages.
    entry_upload_bytes: u64,
    /// One invocation that queries the TLAS, recorded on either side of its
    /// build: the GPU runs neither marker alongside the build, as it would a
    /// pass that does not touch the TLAS, and a pass that does nothing writes
    /// no timestamp on Metal.
    marker_pipeline: wgpu::ComputePipeline,
    /// Its group 3 (`DYNAMIC_TLAS_MARKERS`).
    marker_bind_group: wgpu::BindGroup,
    /// What the markers found, which nothing reads.
    marker_result: wgpu::Buffer,
}

impl DynamicTlas {
    /// An empty dynamic TLAS of `DYNAMIC_TLAS_CAPACITY` entries on
    /// `backend`, built through `queue` at once, so a marker never queries it
    /// unbuilt.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, backend: wgpu::Backend) -> Self {
        let tlas = device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: Some("Dynamic TLAS"),
            max_instances: DYNAMIC_TLAS_CAPACITY,
            // Traced at every lit pixel, as the static TLAS is; ADR 0020's
            // per-frame cost was measured with this flag.
            flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Dynamic TLAS Encoder"),
        });
        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&tlas));
        queue.submit(std::iter::once(encoder.finish()));
        let marker_result = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic TLAS Marker Result"),
            size: size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let layout = DYNAMIC_TLAS_MARKERS.layout(device);
        let marker_bind_group = DYNAMIC_TLAS_MARKERS.bind_group(
            device,
            &layout,
            [
                ("dynamic_tlas", tlas.as_binding()),
                ("marker_result", marker_result.as_entire_binding()),
            ],
        );
        let shader = device.create_shader_module(shaders::DYNAMIC_TLAS_MARKER.descriptor());
        let marker_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("mark_dynamic_tlas"),
            layout: Some(&pipeline_layout(
                device,
                "Dynamic TLAS Marker Pipeline Layout",
                &[(&DYNAMIC_TLAS_MARKERS, &layout)],
            )),
            module: &shader,
            entry_point: Some("mark_dynamic_tlas"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            tlas,
            filled: 0,
            entry_upload_bytes: match backend {
                // `MTLIndirectAccelerationStructureInstanceDescriptor`.
                wgpu::Backend::Metal => 72,
                // `VkAccelerationStructureInstanceKHR`,
                // `D3D12_RAYTRACING_INSTANCE_DESC`.
                _ => 64,
            },
            marker_pipeline,
            marker_bind_group,
            marker_result,
        }
    }

    /// Fills its entries with `frame`'s Moving Instances, each with its
    /// BLAS from `mesh_pool` and its transform from the corner of `anchor`,
    /// the static TLAS's, and records its build into a command buffer of
    /// `device`'s, between markers that write `timestamps`, the end of the
    /// first and the start of the second. Returns the command buffer and the
    /// bytes the build uploads; `None` while it holds none and held none in
    /// the last build, which leaves nothing to build.
    pub fn record(
        &mut self,
        device: &wgpu::Device,
        frame: &FrameData,
        mesh_pool: &MeshPool,
        anchor: [i32; 3],
        timestamps: Option<[wgpu::ComputePassTimestampWrites<'_>; 2]>,
    ) -> Option<(wgpu::CommandBuffer, u64)> {
        // Prepare lists no more than its capacity.
        let listed = frame
            .moving_instances
            .iter()
            .take(DYNAMIC_TLAS_CAPACITY as usize);
        let filled = listed.len();
        let previously_filled = std::mem::replace(&mut self.filled, filled);
        if filled == 0 && previously_filled == 0 {
            return None;
        }
        let mut uploaded_entries = 0;
        for (index, moving) in listed.enumerate() {
            let instance = &frame.instances[moving.instance];
            let blas = mesh_pool.blas(MeshHandle(instance.mesh));
            uploaded_entries += u64::from(blas.is_some());
            self.tlas[index] = blas.map(|blas| tlas_instance(blas, instance, anchor, moving.slot));
        }
        for index in filled..previously_filled {
            self.tlas[index] = None;
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Dynamic TLAS Encoder"),
        });
        let [before, after] = timestamps.map_or([None, None], |writes| writes.map(Some));
        self.mark(&mut encoder, before);
        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&self.tlas));
        self.mark(&mut encoder, after);
        Some((encoder.finish(), uploaded_entries * self.entry_upload_bytes))
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

    /// Its bytes, as `acceleration_structure_bytes` estimates them for its
    /// capacity, whatever it holds.
    pub fn estimated_bytes(
        &self,
        acceleration_structure_bytes: &AccelerationStructureBytes,
    ) -> u64 {
        acceleration_structure_bytes.tlas(self.tlas.get().len())
    }

    /// Bytes of its buffers: the markers' result, as the TLAS's own are
    /// estimated.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.marker_result.size()
    }
}

/// `instance`'s entry in a TLAS: `blas`, placed by its transform from the
/// corner of `anchor` and tagged with its `slot`.
fn tlas_instance(
    blas: &wgpu::Blas,
    instance: &InstanceData,
    anchor: [i32; 3],
    slot: u32,
) -> wgpu::TlasInstance {
    // Every ray's cull mask.
    wgpu::TlasInstance::new(blas, anchored_transform(instance, anchor), slot, 0xff)
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
    use crate::assets::ARENA_CAPACITY_BYTES;
    use crate::assets::pools::tests::{logged, mesh, trace_down, triangle_about_x};
    use crate::gpu::GpuContext;
    use crate::render::instance::padded_columns;
    use crate::render::prepare::InstanceWrite;
    use crate::render::{CameraUniform, LoadStaging, NO_MATERIAL, NO_MESH};

    /// The camera's Anchor in the first frame: 100 km out, where an
    /// Instance's World position would round in f32 and its offset from the
    /// anchor does not.
    const LOAD_ANCHOR: [i32; 3] = [1562, 0, -3];

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
            let mut mesh_pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
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
                mesh: self.mesh.0,
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

    #[test]
    fn the_static_tlas_holds_each_static_instance_in_its_slot_from_the_load_anchor() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        // The first Frame data builds it, even with only Moving Instances.
        let mut empty = StaticTlas::new(&triangle.context.device, 8);
        let moving = triangle.instance(LOAD_ANCHOR, [0.0; 3], glam::Mat3::IDENTITY);
        assert!(triangle.update(&mut empty, LOAD_ANCHOR, vec![(0, moving, true)]));
        assert_eq!(
            trace_down(&triangle.context, &empty.tlas, &[[0.8, 0.0, 0.0]]),
            [None]
        );
        assert!(!triangle.update(&mut empty, LOAD_ANCHOR, vec![(0, moving, true)]));

        let mut tlas = StaticTlas::new(&triangle.context.device, 8);
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

    #[test]
    fn static_instances_past_its_capacity_are_left_out_counted_and_warned_about_once() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        let mut tlas = StaticTlas::new(&triangle.context.device, 2);
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
