//! The BLAS refit: each frame, once the bend pass has bent the Meshes loaded
//! as `MeshUse::Refitted`, their BLASes are brought to the bent shapes before
//! the dynamic TLAS is built over them (#209). `CrowdForm::Refit` refits each
//! in place through wgpu-hal, on Metal only, since wgpu-core rebuilds BLASes
//! but never updates them; `CrowdForm::Rebuild` rebuilds each through wgpu,
//! the shapes a refit must match.
//!
//! wgpu-core keeps refitted BLASes as it built them: each keeps its handle,
//! its build index and its place in the TLAS dependencies that make it
//! resident, so a refit changes only what the BLAS holds.

use crate::assets::MeshPool;
use crate::render::CrowdForm;

/// The BLAS refit's timing in the GPU stats: from the end of the bend pass
/// to the start of the marker after the refits.
pub const BLAS_REFIT_PASS: &str = "blas_refit";

/// Brings bent Meshes' BLASes to their bent shapes in the frame's own
/// submission, ahead of the dynamic TLAS's build.
pub struct BlasRefit {
    form: CrowdForm,
    /// Each refitted BLAS's region, in Mesh pool order, and the buffer
    /// they share, created for the BLASes the first refit finds: one loaded
    /// later is not refitted, as logged.
    #[cfg(target_vendor = "apple")]
    scratch: Option<(wgpu::Buffer, Vec<u64>)>,
    /// What orders the dynamic TLAS's build after the update on Metal, or
    /// None where the build waits for it on the CPU instead.
    #[cfg(target_vendor = "apple")]
    order: Option<metal::Order>,
    /// The marker that ends the refits' time on the GPU, for the refit form.
    #[cfg(target_vendor = "apple")]
    marker: Option<metal::RefitMarker>,
    /// Whether a refit was refused and logged, so it is logged once.
    #[cfg(target_vendor = "apple")]
    refusal_logged: bool,
    /// Whether the build must wait for the update on the CPU: on Metal,
    /// where nothing orders a TLAS build after the BLASes it reads are
    /// written (gfx-rs/wgpu#9215), when no shared event could be made.
    waits_on_cpu: bool,
}

impl BlasRefit {
    /// The update `form` takes on `backend`, through `device` and `queue`;
    /// None for `CrowdForm::Proxies`, and for a form the backend cannot run,
    /// as logged.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        backend: wgpu::Backend,
        form: CrowdForm,
    ) -> Option<Self> {
        if form == CrowdForm::Proxies {
            return None;
        }
        if let Some(reason) = form.unavailable_on(backend) {
            tracing::error!(?form, reason, "BLASes not updated");
            return None;
        }
        #[cfg(target_vendor = "apple")]
        let order = (backend == wgpu::Backend::Metal)
            .then(|| metal::Order::new(device, queue))
            .flatten();
        #[cfg(not(target_vendor = "apple"))]
        let _ = (device, queue);
        #[cfg(target_vendor = "apple")]
        let waits_on_cpu = backend == wgpu::Backend::Metal && order.is_none();
        #[cfg(not(target_vendor = "apple"))]
        let waits_on_cpu = false;
        if waits_on_cpu {
            tracing::warn!("no shared event: each frame waits on the CPU for its BLAS updates");
        }
        Some(Self {
            form,
            #[cfg(target_vendor = "apple")]
            scratch: None,
            #[cfg(target_vendor = "apple")]
            order,
            #[cfg(target_vendor = "apple")]
            marker: (form == CrowdForm::Refit).then(|| metal::RefitMarker::new(device)),
            #[cfg(target_vendor = "apple")]
            refusal_logged: false,
            waits_on_cpu,
        })
    }

    /// Records the update of `mesh_pool`'s refitted BLASes after `head`,
    /// the frame's commands up to its bend pass, submits both through
    /// `queue`, and orders what `queue` submits next after them. A refit
    /// is followed by a marker that writes `refit_end`; returns whether it
    /// was.
    pub fn submit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut head: wgpu::CommandEncoder,
        mesh_pool: &MeshPool,
        refit_end: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) -> bool {
        if self.form == CrowdForm::Rebuild {
            mesh_pool.rebuild_refitted_blases(&mut head);
        }
        let head = head.finish();
        #[cfg(target_vendor = "apple")]
        if let Some(order) = &mut self.order {
            order.signal_from_next_submit(queue);
        }
        #[cfg(target_vendor = "apple")]
        let (submission, refit_timed) = if self.form == CrowdForm::Refit {
            let refits = self.record_refits(device, mesh_pool);
            let marker = refit_end
                .zip(self.marker.as_mut())
                .zip(self.scratch.as_ref())
                .map(|((writes, marker), (scratch, _))| marker.record(device, scratch, writes));
            let refit_timed = marker.is_some();
            (
                queue.submit([head, refits].into_iter().chain(marker)),
                refit_timed,
            )
        } else {
            (queue.submit(std::iter::once(head)), false)
        };
        #[cfg(not(target_vendor = "apple"))]
        let (submission, refit_timed) = {
            let _ = refit_end;
            (queue.submit(std::iter::once(head)), false)
        };
        #[cfg(target_vendor = "apple")]
        if let Some(order) = &self.order {
            order.wait_in_next_submit(queue);
        }
        if self.waits_on_cpu {
            let wait = wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            };
            if let Err(error) = device.poll(wait) {
                tracing::error!(%error, "waiting for BLAS updates failed");
            }
        }
        refit_timed
    }

    /// Bytes of its scratch buffer and its marker's.
    pub fn gpu_memory_bytes(&self) -> u64 {
        #[cfg(target_vendor = "apple")]
        {
            let scratch = self
                .scratch
                .as_ref()
                .map_or(0, |(scratch, _)| scratch.size());
            let marker = self
                .marker
                .as_ref()
                .map_or(0, metal::RefitMarker::gpu_memory_bytes);
            scratch + marker
        }
        #[cfg(not(target_vendor = "apple"))]
        0
    }

    /// A command buffer of `device`'s, encoded through wgpu-hal alone, that
    /// refits each of `mesh_pool`'s refitted BLASes in place from what its
    /// Arena ranges hold.
    #[cfg(target_vendor = "apple")]
    fn record_refits(
        &mut self,
        device: &wgpu::Device,
        mesh_pool: &MeshPool,
    ) -> wgpu::CommandBuffer {
        use wgpu::hal::{self, CommandEncoder as _, api::Metal};

        use crate::assets::pools::REFITTED_BLAS_FLAGS;

        let blases = mesh_pool.refitted_blases();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("BLAS Refit Encoder"),
        });
        let logged = &mut self.refusal_logged;
        let mut refuse = |refusal: &str| {
            if !std::mem::replace(logged, true) {
                tracing::error!(refusal, loaded = blases.len(), "BLASes not refitted");
            }
        };
        if self.scratch.is_none() {
            self.scratch = metal::scratch(device, mesh_pool);
        }
        let Some((scratch, offsets)) = &self.scratch else {
            refuse("no scratch: an Arena is not Metal's");
            return encoder.finish();
        };
        if blases.len() > offsets.len() {
            refuse("those loaded after the first refit have no scratch");
        }
        // SAFETY: wgpu-hal's refit is sound under these invariants, each
        // held here:
        // - Every handle is Metal's: `new` refuses the refit off Metal.
        // - Every wgpu-hal object outlives its use past its guard
        //   (`metal::unguarded`): the Mesh pool, this refit and `blases`
        //   hold their handles throughout, nothing destroys them, and only
        //   the main thread calls into wgpu meanwhile (ADR 0016).
        // - The encoder is used through wgpu-hal alone, never ended here,
        //   and finished by wgpu; wgpu-core panics on a mix of the two.
        // - The GPU's use ends before any object is freed: wgpu-core never
        //   sees what this command buffer uses, but wgpu-hal's Metal command
        //   buffers retain every resource their commands name until they
        //   complete (its `retain_command_buffer_references`, on by
        //   default).
        // - Each refit's descriptor is the one wgpu-core built the BLAS from
        //   (`MeshPool::load_for`), with its flags, which allow updates: the
        //   same buffers, offsets, counts, stride and formats.
        // - Each refit has its own scratch region, of the size Metal asks
        //   for its update, in a buffer any usage makes on Metal. The next
        //   frame's refits reuse the regions, and the frames before them may
        //   still run: Metal tracks hazards on wgpu-hal's buffers, scratch
        //   included, so they wait.
        // - The bend pass's writes to the vertex Arena come first in the
        //   queue, and Metal tracks hazards on wgpu-hal's buffers, so each
        //   refit reads the bent shapes; the TLAS build that reads the
        //   BLASes waits for this command buffer through `metal::Order`, or
        //   on the CPU.
        // - wgpu-core never learns of the refit, and needs not: it changes no
        //   BLAS's size, handle or build index, so the TLAS built next still
        //   validates against it, and wgpu-core's barriers are no-ops on
        //   Metal.
        unsafe {
            let (Some(vertices), Some(indices), Some(scratch)) = (
                metal::unguarded(mesh_pool.vertex_arena().as_hal::<Metal>()),
                metal::unguarded(mesh_pool.index_arena().as_hal::<Metal>()),
                metal::unguarded(scratch.as_hal::<Metal>()),
            ) else {
                refuse("an Arena or the scratch is not Metal's");
                return encoder.finish();
            };
            let mut handles: Vec<wgpu::Blas> =
                blases.iter().map(|blas| blas.blas.clone()).collect();
            let Some(raw_blases) = handles
                .iter_mut()
                .map(|blas| metal::unguarded(blas.as_hal::<Metal>()))
                .collect::<Option<Vec<_>>>()
            else {
                refuse("a BLAS is not Metal's");
                return encoder.finish();
            };
            let entries: Vec<_> = blases
                .iter()
                .map(|blas| metal::triangles(blas, vertices, indices))
                .collect();
            let descriptors: Vec<_> = entries
                .iter()
                .zip(&raw_blases)
                .zip(offsets.iter())
                .map(
                    |((entries, raw_blas), &offset)| hal::BuildAccelerationStructureDescriptor {
                        entries,
                        mode: hal::AccelerationStructureBuildMode::Update,
                        flags: REFITTED_BLAS_FLAGS,
                        source_acceleration_structure: Some(*raw_blas),
                        destination_acceleration_structure: *raw_blas,
                        scratch_buffer: scratch,
                        scratch_buffer_offset: offset,
                    },
                )
                .collect();
            encoder.as_hal_mut::<Metal, _, _>(|hal_encoder| match hal_encoder {
                Some(hal_encoder) => {
                    hal_encoder.build_acceleration_structures(descriptors.len() as u32, descriptors)
                }
                None => refuse("the encoder is not Metal's"),
            });
        }
        encoder.finish()
    }
}

/// What only Metal needs: refit descriptors in wgpu-hal's terms, their
/// scratch, and the order between a refit and the TLAS build after it.
#[cfg(target_vendor = "apple")]
mod metal {
    use wgpu::hal::{self, Device as _, api::Metal};

    use crate::assets::MeshPool;
    use crate::assets::pools::{MeshBlas, REFITTED_BLAS_FLAGS};
    use crate::cooked::CookedVertex;
    use crate::render::bind_groups::{BLAS_REFIT_MARKERS, pipeline_layout};
    use crate::render::shaders;

    /// Scratch regions start this far apart at least: a margin over the
    /// one-byte alignment wgpu-hal reports for Metal.
    const SCRATCH_ALIGNMENT: u64 = 256;

    /// The wgpu-hal object `guard` reads, kept past the guard: each guard
    /// holds its device's snatch lock, which a thread must not take twice,
    /// and a refit needs several objects at once.
    ///
    /// # Safety
    ///
    /// The resource the object belongs to is neither destroyed nor dropped
    /// while the reference lives.
    pub(super) unsafe fn unguarded<'a, T>(
        guard: Option<impl std::ops::Deref<Target = T>>,
    ) -> Option<&'a T> {
        // SAFETY: wgpu-core boxes the object in the resource, so its address
        // holds while the resource lives undestroyed, as the caller ensures.
        guard.map(|guard| unsafe { &*std::ptr::from_ref(&*guard) })
    }

    /// `blas`'s triangles as wgpu-core gave them to its build, from
    /// `vertices` and `indices`, the Mesh pool's Arenas.
    pub(super) fn triangles<'a>(
        blas: &MeshBlas,
        vertices: &'a hal::metal::Buffer,
        indices: &'a hal::metal::Buffer,
    ) -> hal::AccelerationStructureEntries<'a, hal::metal::Buffer> {
        let geometry = &blas.geometry;
        let index_count = geometry.index_count.unwrap_or_default();
        hal::AccelerationStructureEntries::Triangles(vec![hal::AccelerationStructureTriangles {
            vertex_buffer: Some(vertices),
            vertex_format: geometry.vertex_format,
            first_vertex: blas.first_vertex,
            vertex_count: geometry.vertex_count,
            vertex_stride: size_of::<CookedVertex>() as u64,
            indices: Some(hal::AccelerationStructureTriangleIndices {
                format: wgpu::IndexFormat::Uint32,
                buffer: Some(indices),
                offset: blas.first_index * size_of::<u32>() as u32,
                count: index_count,
            }),
            transform: None,
            flags: geometry.flags,
        }])
    }

    /// A scratch buffer of `device`'s with a region for each of
    /// `mesh_pool`'s refitted BLASes, as large as Metal asks for its update,
    /// and the regions' offsets; None if a handle is not Metal's.
    pub(super) fn scratch(
        device: &wgpu::Device,
        mesh_pool: &MeshPool,
    ) -> Option<(wgpu::Buffer, Vec<u64>)> {
        // SAFETY: the handles are Metal's, as `BlasRefit::new` checked, and
        // the Mesh pool holds the Arenas past this call (`unguarded`).
        let sizes: Vec<u64> = unsafe {
            let hal_device = device.as_hal::<Metal>()?;
            let vertices = unguarded(mesh_pool.vertex_arena().as_hal::<Metal>())?;
            let indices = unguarded(mesh_pool.index_arena().as_hal::<Metal>())?;
            mesh_pool
                .refitted_blases()
                .iter()
                .map(|blas| {
                    let entries = triangles(blas, vertices, indices);
                    let sizes = hal_device.get_acceleration_structure_build_sizes(
                        &hal::GetAccelerationStructureBuildSizesDescriptor {
                            entries: &entries,
                            flags: REFITTED_BLAS_FLAGS,
                        },
                    );
                    sizes
                        .update_scratch_size
                        .next_multiple_of(SCRATCH_ALIGNMENT)
                })
                .collect()
        };
        let offsets = sizes
            .iter()
            .scan(0, |next, size| Some(std::mem::replace(next, *next + size)))
            .collect();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("BLAS Refit Scratch"),
            size: sizes.iter().sum::<u64>().max(SCRATCH_ALIGNMENT),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        Some((buffer, offsets))
    }

    /// One invocation that reads the refits' scratch, recorded after them in
    /// a command buffer of its own: Metal tracks hazards on the scratch, so
    /// the GPU runs it once they are done, and a pass that does nothing
    /// writes no timestamp on Metal (#204).
    pub(super) struct RefitMarker {
        pipeline: wgpu::ComputePipeline,
        layout: wgpu::BindGroupLayout,
        /// What it read, which nothing reads.
        result: wgpu::Buffer,
        /// Its group 3 (`BLAS_REFIT_MARKERS`), made for the scratch the
        /// first refit makes, which no later refit replaces.
        bind_group: Option<wgpu::BindGroup>,
    }

    impl RefitMarker {
        pub(super) fn new(device: &wgpu::Device) -> Self {
            let layout = BLAS_REFIT_MARKERS.layout(device);
            let shader = device.create_shader_module(shaders::BLAS_REFIT_MARKER.descriptor());
            let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("mark_blas_refit"),
                layout: Some(&pipeline_layout(
                    device,
                    "BLAS Refit Marker Pipeline Layout",
                    &[(&BLAS_REFIT_MARKERS, &layout)],
                )),
                module: &shader,
                entry_point: Some("mark_blas_refit"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
            let result = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("BLAS Refit Marker Result"),
                size: size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            Self {
                pipeline,
                layout,
                result,
                bind_group: None,
            }
        }

        /// A command buffer of `device`'s that reads `scratch`, timed into
        /// `timestamp_writes`.
        pub(super) fn record(
            &mut self,
            device: &wgpu::Device,
            scratch: &wgpu::Buffer,
            timestamp_writes: wgpu::ComputePassTimestampWrites<'_>,
        ) -> wgpu::CommandBuffer {
            let bind_group = self.bind_group.get_or_insert_with(|| {
                BLAS_REFIT_MARKERS.bind_group(
                    device,
                    &self.layout,
                    [
                        ("refit_scratch", scratch.as_entire_binding()),
                        ("marker_result", self.result.as_entire_binding()),
                    ],
                )
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("BLAS Refit Marker Encoder"),
            });
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("BLAS Refit Marker"),
                timestamp_writes: Some(timestamp_writes),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(BLAS_REFIT_MARKERS.index, &*bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
            drop(pass);
            encoder.finish()
        }

        pub(super) fn gpu_memory_bytes(&self) -> u64 {
            self.result.size()
        }
    }

    /// Orders the GPU work of a submission after the one before it: the
    /// earlier signals a shared event as it completes, and wgpu-hal's
    /// strict event sync holds every command buffer encoded after it until
    /// the event is signalled. Metal orders no TLAS build after the BLASes
    /// it reads are written (gfx-rs/wgpu#9215), and waiting on the CPU
    /// would stall the main thread on the GPU each frame.
    pub(super) struct Order {
        device: wgpu::Device,
        /// wgpu-hal's fence, for its shared event; None once destroyed.
        fence: Option<hal::metal::Fence>,
        /// The value the last submission signals.
        signalled: u64,
    }

    impl Order {
        /// The order on `device`'s `queue`, whose strict event sync it
        /// turns on for the queue's lifetime; None, as logged, if Metal
        /// makes no shared event.
        pub(super) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
            // SAFETY: both handles are Metal's; the fence is destroyed by
            // the device that made it (`Drop`), and the queue lives as long
            // as wgpu's. Strict event sync then adds a wait to every command
            // buffer and a command buffer to every submission on the queue.
            let order = Self {
                device: device.clone(),
                fence: Some(unsafe { device.as_hal::<Metal>()?.create_fence().ok()? }),
                signalled: 0,
            };
            // Dropping `order` destroys the fence on the way out.
            order.fence.as_ref()?.raw_shared_event()?;
            let queue = unsafe { queue.as_hal::<Metal>()? };
            if let Err(error) = queue.enable_strict_event_sync() {
                tracing::error!(%error, "strict event sync not enabled");
                return None;
            }
            Some(order)
        }

        /// Has the submission `queue` makes next signal a new value as it
        /// completes: call it right before that submission.
        pub(super) fn signal_from_next_submit(&mut self, queue: &wgpu::Queue) {
            self.signalled += 1;
            self.stage(queue, true);
        }

        /// Has the submission `queue` makes next wait for the last value
        /// signalled, ahead of every command buffer encoded since the last
        /// submission: call it right after the submission that signals,
        /// before encoding what must follow it.
        pub(super) fn wait_in_next_submit(&self, queue: &wgpu::Queue) {
            self.stage(queue, false);
        }

        /// Stages a signal of the last value, or a wait for it, on `queue`.
        fn stage(&self, queue: &wgpu::Queue, signal: bool) {
            let Some(event) = self
                .fence
                .as_ref()
                .and_then(|fence| fence.raw_shared_event())
            else {
                return;
            };
            // SAFETY: the queue is Metal's. The submission right after the
            // staging takes it, whichever it is: only the main thread submits
            // (ADR 0016), and `Renderer::render` submits nothing between the
            // update's submission and the dynamic TLAS's build.
            unsafe {
                let Some(queue) = queue.as_hal::<Metal>() else {
                    return;
                };
                if signal {
                    queue.add_signal_event(event.into(), self.signalled);
                } else {
                    queue.add_wait_event(event.into(), self.signalled);
                }
            }
        }
    }

    impl Drop for Order {
        fn drop(&mut self) {
            // SAFETY: the device made the fence, and no CPU wait reads it.
            // Submissions still pending signal or wait on its shared event,
            // which each staging retained, so the event outlives them.
            unsafe {
                if let (Some(device), Some(fence)) =
                    (self.device.as_hal::<Metal>(), self.fence.take())
                {
                    device.destroy_fence(fence);
                }
            }
        }
    }
}
