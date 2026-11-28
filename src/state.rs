use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "cook")]
use anyhow::Context;

use crate::assets::PoolLoader;
use crate::cooked::{COOKED_CACHE_DIRECTORY, CookedCache};
use crate::gpu::GpuContext;
use crate::render::{FramePreparer, RenderSwitches, Renderer};
use crate::scenes::ReferenceScene;
use crate::scenes::simulation::Simulation;
use crate::stats::{FrameRecorder, FrameStats, GpuFrameStats};
use crate::streaming::ChunkStreaming;

/// The Scene's Simulation, the Frame data prepared from it, and the Renderer
/// that draws that into a target view.
pub struct State {
    pub(crate) simulation: Simulation,
    pub(crate) streaming: ChunkStreaming,
    recorder: FrameRecorder,
    preparer: FramePreparer,
    renderer: Renderer,
    /// The sources the launch's cook found absent, which a failed load names.
    absent_sources: Vec<String>,
}

impl State {
    /// A State that renders `scene`, moved with its camera by `shift` metres,
    /// into `width` × `height` views of `target_format` textures, both
    /// non-zero, seen from the scene's camera at frame 0, as `switches` set.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scene: ReferenceScene,
        shift: glam::DVec3,
        switches: RenderSwitches,
    ) -> anyhow::Result<State> {
        let _span = tracing::info_span!("state_init").entered();
        if let Some(reason) = switches.crowd_form.unavailable_on(context.backend()) {
            anyhow::bail!("{:?} is unavailable: {reason}", switches.crowd_form);
        }

        let mut renderer = Renderer::new(context, target_format, width, height, switches);
        if scene == ReferenceScene::World {
            renderer.grow_frame_staging_to_largest();
            renderer.keep_load_staging_slots();
        }
        let load_bytes_per_frame = renderer.capacities().load_bytes_per_frame;
        let Renderer {
            device,
            mesh_pool,
            material_pool,
            load_staging,
            ..
        } = &mut renderer;

        let recorder = FrameRecorder::new();
        let cache = Arc::new(CookedCache::new(COOKED_CACHE_DIRECTORY));
        let mut streaming = ChunkStreaming::new(
            scene,
            shift,
            scene.load_radius_metres(),
            load_bytes_per_frame,
            cache.clone(),
        )?;
        let reader = streaming.reader();
        let mut loader = PoolLoader {
            device,
            staging: load_staging,
            reader: &reader,
            mesh_pool,
            material_pool,
        };
        // A dev launch cooks each Asset it loads that is missing or stale, as
        // it loads it (ADR 0018); a load that then fails names the sources it
        // found absent.
        #[cfg(feature = "cook")]
        let mut loader = crate::cook::CookingLoader::new(
            scene,
            std::path::Path::new(crate::cook::ASSETS_DIRECTORY),
            &cache,
            crate::cook::CookSettings::default(),
            &mut loader,
        )
        .with_context(|| format!("reading {}'s sources", scene.name()))?;
        #[cfg(feature = "cook")]
        let absent_sources = loader.absent_sources().to_vec();
        #[cfg(not(feature = "cook"))]
        let absent_sources: Vec<String> = Vec::new();

        let mut simulation =
            Simulation::spawned(scene, shift, switches.crowd_form, &mut loader, &recorder)
                .map_err(|error| naming_absent_sources(error, &absent_sources))?;
        #[cfg(feature = "cook")]
        {
            let mut launch_cook = loader.into_cook();
            let cache = cache.clone();
            streaming.cook_each_chunk(Box::new(move |id| launch_cook.cook_for_scene(&cache, id)));
        }
        let preparer = FramePreparer::new(&mut simulation.world, renderer.capacities());

        Ok(Self {
            simulation,
            streaming,
            recorder,
            preparer,
            renderer,
            absent_sources,
        })
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    /// Moves the camera by `frame_delta`, runs the Ticks it completes,
    /// streams the Chunks around the camera, prepares the frame's Frame data,
    /// blended between the two newest Ticks, and uploads it.
    pub fn update(&mut self, frame_delta: Duration) {
        if let Err(error) = self.advance(frame_delta, false) {
            tracing::error!(error = format!("{error:#}"), "Chunk streaming failed");
        }
    }

    /// `update`, waiting until the Chunk streamer reports the frame Settled;
    /// fails when a load fails.
    pub(crate) fn update_settled(&mut self, frame_delta: Duration) -> anyhow::Result<()> {
        self.advance(frame_delta, true)
            .map_err(|error| naming_absent_sources(error, &self.absent_sources))
    }

    fn advance(&mut self, frame_delta: Duration, settle: bool) -> anyhow::Result<()> {
        self.recorder.begin();
        self.recorder.counters.dropped_time = self.simulation.simulate(frame_delta);

        let started = Instant::now();
        let streamed =
            self.streaming
                .update(&mut self.simulation.world, &mut self.renderer, settle)?;
        self.recorder.counters.late_chunks = streamed.late_chunks;
        self.recorder.counters.late_far_proxies = streamed.late_far_proxies;
        self.recorder.counters.chunks_loaded = streamed.chunks_loaded;
        self.recorder.counters.chunks_unloaded = streamed.chunks_unloaded;
        self.recorder.counters.bytes_read = streamed.bytes_read;
        self.recorder.counters.uploaded_bytes += streamed.written_bytes;
        self.recorder.counters.pool_entries_freed = streamed.pool_entries_freed;
        self.recorder.counters.meshes_refused = streamed.meshes_refused;
        self.recorder.counters.materials_refused = streamed.materials_refused;
        self.recorder.counters.textures_refused = streamed.textures_refused;
        self.recorder.counters.chunks_not_loaded = streamed.chunks_not_loaded;
        self.recorder.counters.late_mips = streamed.late_mips;
        self.recorder.counters.holding = streamed.holding;
        self.recorder.counters.fast_travel_hold = streamed.fast_travel_hold;
        self.recorder.counters.far_field_filling = streamed.far_field_filling;
        self.recorder.counters.far_field_fill = streamed.far_field_fill;
        self.recorder.stage("stream_chunks", started);

        let started = Instant::now();
        // Writes a failed upload lost are written again, with every Instance.
        if self.renderer.take_lost_instance_writes() {
            self.preparer.rewrite_all_instances();
        }
        let frame = self.preparer.prepare(
            &mut self.simulation.world,
            self.renderer.mesh_pool.meshes(),
            self.renderer.material_pool.generations(),
            self.renderer.aspect_ratio(),
        );
        self.recorder.stage("prepare_frame", started);

        self.renderer.upload(frame, &mut self.recorder);
        self.recorder.counters.renderables_refused = self.preparer.renderables_refused();
        self.recorder.counters.lights_overflowed = self.preparer.lights_overflowed();
        self.recorder.counters.moving_instances_left_out =
            self.preparer.moving_instances_left_out();
        self.recorder.counters.bends_left_out = self.preparer.bends_left_out();
        Ok(())
    }

    /// The Cooked cache's `content_hash`: the Blobs the scene loaded so far.
    pub(crate) fn scene_content_hash(&self) -> String {
        self.streaming.content_hash()
    }

    /// Draws the frame into `target`, a view in the format and size this
    /// State renders to. Returns the frame's submission.
    pub fn render(&mut self, target: &wgpu::TextureView) -> wgpu::SubmissionIndex {
        let submission = self
            .renderer
            .render(self.preparer.frame(), target, &mut self.recorder);
        self.recorder.end();
        submission
    }

    /// The shadow mask the last frame rendered wrote.
    pub(crate) fn shadow_mask(&self) -> &wgpu::Texture {
        self.renderer.shadow_mask()
    }

    /// The irradiance the Probes hold, with the `probes` Render switch on.
    pub(crate) fn probe_irradiance(&self) -> Option<&wgpu::Texture> {
        self.renderer.probe_irradiance()
    }

    /// The depth moments the Probes hold, with the `probes` Render switch on.
    pub(crate) fn probe_depth_moments(&self) -> Option<&wgpu::Texture> {
        self.renderer.probe_depth_moments()
    }

    /// One past the last frame whose submission completed, as far as the
    /// submissions and polls since have seen.
    #[cfg(test)]
    pub(crate) fn frames_completed(&self) -> u64 {
        self.renderer.frames_completed()
    }

    /// Makes each `render` wait `pause` before it submits the frame's
    /// encoder, after the frame's earlier submissions.
    #[cfg(test)]
    pub(crate) fn pause_before_frame_submit(&mut self, pause: Duration) {
        self.renderer.pause_before_frame_submit = pause;
    }

    /// The GPU stats of earlier frames that the last `update` or
    /// `flush_gpu_frame_stats` read back, by frame number, counted from 0.
    pub fn gpu_frame_stats(&self) -> &[GpuFrameStats] {
        self.renderer.gpu_frame_stats()
    }

    /// Waits until the GPU stats of every frame rendered are read back,
    /// for `gpu_frame_stats`: the end of a run.
    pub fn flush_gpu_frame_stats(&mut self) -> anyhow::Result<()> {
        self.renderer.flush_gpu_frame_stats()
    }

    /// The stats of the last frame, from `update` through `render`.
    pub fn frame_stats(&self) -> FrameStats {
        let gpu_memory_bytes_by_kind = self.renderer.gpu_memory_bytes_by_kind();
        FrameStats {
            gpu_memory_bytes: gpu_memory_bytes_by_kind.iter().sum(),
            gpu_memory_bytes_by_kind,
            arena_bytes_in_use: self.renderer.mesh_pool.arena_bytes_in_use(),
            allocator_report_bytes: self.renderer.allocator_report_bytes(),
            ..self.recorder.stats()
        }
    }
}

/// `error`, naming the `absent` sources when there are any.
fn naming_absent_sources(error: anyhow::Error, absent: &[String]) -> anyhow::Error {
    match absent {
        [] => error,
        absent => error.context(format!("absent sources: {}", absent.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    use super::*;
    use crate::assets::MeshHandle;
    use crate::camera::Camera;
    use crate::components::{Bend, Transform};
    use crate::input::{Input, clear_just_pressed};
    use crate::scenes::simulation::simulation_schedules;
    use crate::streaming::LoadedChunks;
    #[cfg(feature = "cook")]
    use crate::streaming::ResidentFarProxies;
    use crate::time::{FixedTime, TICK, TICK_SECONDS, Time, run_ticks};
    use crate::transform::{
        PreviousWorldTransform, Teleported, WorldTransform, propagate_world_transforms,
        reset_teleported,
    };

    #[derive(Resource, Default)]
    struct SystemThreads(Vec<std::thread::ThreadId>);

    fn record_thread(mut threads: ResMut<SystemThreads>) {
        threads.0.push(std::thread::current().id());
    }

    #[test]
    fn simulation_schedules_run_single_threaded() {
        let (mut frame_schedule, mut fixed_schedule) = simulation_schedules(&FrameRecorder::new());
        let mut world = World::new();
        world.insert_resource(Input::default());
        world.insert_resource(Time::default());
        world.insert_resource(FixedTime::default());
        world.insert_resource(Camera::new());
        world.insert_resource(SystemThreads::default());
        for schedule in [&mut frame_schedule, &mut fixed_schedule] {
            schedule.add_systems(record_thread);
            schedule.run(&mut world);
        }
        // The multi-threaded executor runs a `Send` system that is not exclusive on a thread of the
        // process-global pool, never the caller's [ref:bevy-ecs-0.20.1-multi-threaded-executor-spawns-on-the-pool].
        assert_eq!(
            world.resource::<SystemThreads>().0,
            [std::thread::current().id(); 2]
        );
    }

    /// 100 km out, so the blend is checked at Game-world scale.
    const PIVOT: glam::DVec3 = glam::dvec3(100_000.25, 1.5, -3.0);
    const FAR_PIVOT: glam::DVec3 = glam::dvec3(105_000.25, 1.5, -3.0);
    const ORBIT_RADIUS: f64 = 2.5;
    const RADIANS_PER_SECOND: f64 = 1.0;
    /// The Tick that spawns or teleports.
    const JUMP_TICK: u64 = 100;

    /// Where a body circling `pivot` about +Y from +X is after `ticks` Ticks.
    fn orbit_position(pivot: glam::DVec3, ticks: u64) -> glam::DVec3 {
        let angle = RADIANS_PER_SECOND * ticks as f64 * TICK_SECONDS;
        pivot + ORBIT_RADIUS * glam::dvec3(angle.cos(), 0.0, -angle.sin())
    }

    /// The analytic blend, `weight` of the way from Tick `ticks - 1` to Tick
    /// `ticks`, of a body on the orbit around `pivot` since Tick `since`.
    fn blend_on_orbit(pivot: glam::DVec3, ticks: u64, weight: f64, since: u64) -> glam::DVec3 {
        let previous = orbit_position(pivot, ticks.saturating_sub(1).max(since));
        previous + (orbit_position(pivot, ticks) - previous) * weight
    }

    #[derive(Component)]
    struct Pivot;

    /// Turns the pivot about +Y from the Tick count, as a game-state system.
    fn turn_pivot(fixed_time: Res<FixedTime>, mut pivot: Single<&mut Transform, With<Pivot>>) {
        let angle = RADIANS_PER_SECOND * (fixed_time.tick + 1) as f64 * TICK_SECONDS;
        pivot.rotation = glam::Quat::from_rotation_y(angle as f32);
    }

    /// The State's fixed schedule turning a pivot at `PIVOT`, and an ECS
    /// World with that pivot and a body on its orbit, settled as `State::new`
    /// leaves it. Returns the pivot and the body too.
    fn orbit() -> (World, Schedule, Entity, Entity) {
        let mut world = World::new();
        world.insert_resource(Input::default());
        world.insert_resource(FixedTime::default());
        let pivot = world
            .spawn((
                Transform::from_translation(PIVOT),
                Pivot,
                PreviousWorldTransform::default(),
            ))
            .id();
        let body = world
            .spawn((
                Transform::from_translation(glam::DVec3::X * ORBIT_RADIUS),
                ChildOf(pivot),
                PreviousWorldTransform::default(),
            ))
            .id();
        world.run_system_once(propagate_world_transforms).unwrap();
        let (_, mut fixed_schedule) = simulation_schedules(&FrameRecorder::new());
        fixed_schedule.add_systems(
            turn_pivot
                .after(clear_just_pressed)
                .before(reset_teleported),
        );
        (world, fixed_schedule, pivot, body)
    }

    /// Runs 200 frames of 144, 60, 30 and 90 Hz in turn, which run 0 to 3
    /// Ticks each, and after each calls `check` with the Ticks run and the
    /// blend weight, both from the frames' summed time.
    fn run_frames(
        world: &mut World,
        fixed_schedule: &mut Schedule,
        mut check: impl FnMut(&World, u64, f64),
    ) {
        let frame_deltas =
            [6_944_444, 16_666_667, 33_333_333, 11_111_111].map(Duration::from_nanos);
        let mut elapsed = Duration::ZERO;
        for frame_delta in frame_deltas.into_iter().cycle().take(200) {
            run_ticks(fixed_schedule, world, frame_delta);
            elapsed += frame_delta;
            let ticks = elapsed.as_nanos() / TICK.as_nanos();
            let weight = (elapsed.as_nanos() % TICK.as_nanos()) as f64 / TICK.as_nanos() as f64;
            check(world, ticks as u64, weight);
        }
    }

    /// `entity` is drawn, as prepare draws it, within 10 µm of `expected`.
    fn assert_drawn_at(world: &World, entity: Entity, expected: glam::DVec3, ticks: u64) {
        let entity = world.entity(entity);
        let blend_weight = world.resource::<FixedTime>().blend_weight;
        let drawn = entity
            .get::<WorldTransform>()
            .unwrap()
            .drawn(entity.get(), blend_weight)
            .translation();
        assert!(
            (drawn - expected).length() <= 1e-5,
            "after {ticks} Ticks drawn at {drawn}, not {expected}"
        );
    }

    #[test]
    fn between_ticks_a_moving_entity_is_drawn_at_the_blend_of_its_two_newest_tick_states() {
        let (mut world, mut fixed_schedule, pivot, body) = orbit();
        run_frames(&mut world, &mut fixed_schedule, |world, ticks, weight| {
            assert_drawn_at(world, pivot, PIVOT, ticks);
            assert_drawn_at(world, body, blend_on_orbit(PIVOT, ticks, weight, 0), ticks);
        });
    }

    /// Sets each bend's curvature to the count of Ticks run, as a
    /// game-state system.
    fn count_ticks(fixed_time: Res<FixedTime>, mut bends: Query<&mut Bend>) {
        for mut bend in &mut bends {
            bend.curvature = (fixed_time.tick + 1) as f32;
        }
    }

    #[test]
    fn between_ticks_a_bend_is_drawn_at_the_blend_of_its_two_newest_tick_curvatures() {
        let mut world = World::new();
        world.insert_resource(Input::default());
        world.insert_resource(FixedTime::default());
        let bend = Bend {
            rest: MeshHandle {
                index: 0,
                generation: 0,
            },
            azimuth: 0.0,
            curvature: 0.0,
            previous_curvature: 0.0,
        };
        let entity = world.spawn(bend).id();
        let (_, mut fixed_schedule) = simulation_schedules(&FrameRecorder::new());
        fixed_schedule.add_systems(
            count_ticks
                .after(clear_just_pressed)
                .before(reset_teleported),
        );
        run_frames(&mut world, &mut fixed_schedule, |world, ticks, weight| {
            let blend_weight = world.resource::<FixedTime>().blend_weight;
            let drawn = world
                .get::<Bend>(entity)
                .unwrap()
                .drawn_curvature(blend_weight);
            // From the spawn's 0 until the first Tick, then a weight of the
            // way from the second newest Tick's count to the newest's.
            let expected = ticks.saturating_sub(1) as f64 + if ticks == 0 { 0.0 } else { weight };
            assert!(
                (f64::from(drawn) - expected).abs() <= 1e-4,
                "after {ticks} Ticks drawn at {drawn}, not {expected}"
            );
        });
    }

    #[derive(Resource)]
    struct Spawned(Entity);

    /// At `JUMP_TICK`, moves the pivot to `FAR_PIVOT` by a jump, and spawns
    /// a body at rest at `PIVOT`, 100 km from where a new World transform
    /// starts.
    fn jump(
        fixed_time: Res<FixedTime>,
        mut commands: Commands,
        pivot: Single<(Entity, &mut Transform), With<Pivot>>,
    ) {
        if fixed_time.tick == JUMP_TICK {
            let (pivot, mut transform) = pivot.into_inner();
            transform.translation = FAR_PIVOT;
            commands.entity(pivot).insert(Teleported);
            let spawned = commands
                .spawn((
                    Transform::from_translation(PIVOT),
                    PreviousWorldTransform::default(),
                ))
                .id();
            commands.insert_resource(Spawned(spawned));
        }
    }

    #[test]
    fn a_spawned_or_teleported_entity_is_never_drawn_between_its_old_and_new_place() {
        let (mut world, mut fixed_schedule, pivot, body) = orbit();
        fixed_schedule.add_systems(jump.after(turn_pivot).before(reset_teleported));
        // Only frames between the jump and the next Tick can draw a sweep.
        let mut frames_after_jump = 0;
        run_frames(&mut world, &mut fixed_schedule, |world, ticks, weight| {
            frames_after_jump += usize::from(ticks == JUMP_TICK + 1);
            // The jump in Tick `JUMP_TICK` is first drawn after it.
            let (pivot_position, since) = if ticks <= JUMP_TICK {
                (PIVOT, 0)
            } else {
                (FAR_PIVOT, JUMP_TICK + 1)
            };
            assert_drawn_at(world, pivot, pivot_position, ticks);
            let expected = blend_on_orbit(pivot_position, ticks, weight, since);
            assert_drawn_at(world, body, expected, ticks);
            if let Some(Spawned(spawned)) = world.get_resource() {
                assert_drawn_at(world, *spawned, PIVOT, ticks);
            }
        });
        assert!(world.contains_resource::<Spawned>());
        assert!(frames_after_jump > 0, "no frame drawn just after the jump");
    }

    /// `shadows`'s Chunk, with a copy of it loaded 40 Chunks (10,240 m) along
    /// +X, draws the same image seen from the copy, once the camera has
    /// moved there, as seen from the origin: with both Chunks kept loaded,
    /// so no static Instance changes, the static TLAS, anchored at the
    /// origin, re-anchors at the camera, and the copy's shadow rays compose
    /// and lift as the origin's did (ADR 0014).
    #[test]
    fn one_chunks_content_draws_alike_at_the_origin_and_ten_km_away() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::Shadows;
        let (width, height) = (640, 360);
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            width,
            height,
            scene,
            glam::DVec3::ZERO,
            RenderSwitches {
                wait_for_static_tlas: true,
                ..RenderSwitches::default()
            },
        )
        .expect("shadows loads");
        state.streaming.set_load_radius(20_000.0);
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Ten Kilometre Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::capture::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let copy_chunks = 40;
        let shift = glam::dvec3(
            f64::from(copy_chunks) * crate::cooked::CHUNK_EDGE_METRES,
            0.0,
            0.0,
        );
        let draw = |state: &mut State, frame: u32, shift| {
            let camera = scene.camera_at(frame, shift);
            state.simulation.world.insert_resource(camera);
            state
                .update_settled(scene.frame_delta())
                .expect("the frame settles");
            state.render(&target_view);
            crate::capture::read_back(&context, &target).expect("the frame reads back")
        };
        draw(&mut state, 0, glam::DVec3::ZERO);
        let mut loaded = state.simulation.world.resource_mut::<LoadedChunks>();
        let origin = &loaded.0[0];
        let mut copy = crate::streaming::LoadedChunk {
            id: crate::cooked::AssetId(!origin.id.0),
            coordinates: [origin.coordinates[0] + copy_chunks, origin.coordinates[1]],
            placements: origin
                .placements
                .iter()
                .map(|placement| crate::streaming::chunks::ChunkPlacement {
                    transform: placement.transform,
                    mesh: placement.mesh,
                    material: placement.material,
                })
                .collect(),
            lights: origin.lights.clone(),
            meshes: Vec::new(),
            materials: Vec::new(),
            held: false,
        };
        copy.shift(shift);
        loaded.0.push(copy);

        let at_origin = (1..4)
            .map(|frame| draw(&mut state, frame, glam::DVec3::ZERO))
            .last();
        let at_copy = (4..8).map(|frame| draw(&mut state, frame, shift)).last();
        let (at_origin, at_copy) = (at_origin.expect("drawn"), at_copy.expect("drawn"));
        let difference = crate::capture::compare_images(&at_origin, &at_copy);
        assert!(
            at_origin == at_copy,
            "seen from 10,240 m, the copy draws otherwise: {} pixels past the Goldens' tolerance, by up to {}",
            difference.differing_pixels,
            difference.max_channel_delta
        );
    }

    /// Flown past with a Load radius of 0 m, a Chunk of `far_coplanar` loads
    /// once the camera is over it and is drawn; is held, so drawn no more,
    /// once the camera is 172 m past it; and unloads 944 m past it. Each
    /// frame waits until it is Settled, and the bench's Counters count it.
    #[test]
    fn a_chunk_loads_is_drawn_held_and_unloaded_as_the_camera_flies_past() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::FarCoplanar;
        let (width, height) = (64, 36);
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            width,
            height,
            scene,
            glam::DVec3::ZERO,
            RenderSwitches::default(),
        )
        .expect("far_coplanar loads");
        state.streaming.set_load_radius(0.0);
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Chunk Flight Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::capture::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let watched = [0, -8];
        let middle_z = -7.5 * crate::cooked::CHUNK_EDGE_METRES;
        let mut frame_stats = Vec::new();
        let mut watched_states = Vec::new();
        for x in [128.0, 428.0, 1200.0] {
            let position = glam::dvec3(x, 50.0, middle_z);
            state
                .simulation
                .world
                .insert_resource(crate::camera::Camera::looking_at(
                    position,
                    position + glam::dvec3(1.0, -10.0, 0.0),
                ));
            state
                .update_settled(scene.frame_delta())
                .expect("the frame settles");
            state.render(&target_view);
            frame_stats.push(state.frame_stats());
            let loaded = state.simulation.world.resource::<LoadedChunks>();
            let watched_chunk = loaded.0.iter().find(|chunk| chunk.coordinates == watched);
            watched_states.push(watched_chunk.map(|chunk| chunk.held));
        }
        state.flush_gpu_frame_stats().expect("GPU stats read back");

        assert_eq!(watched_states, [Some(false), Some(true), None]);
        let loads = frame_stats.iter().map(|stats| stats.chunks_loaded);
        let unloads = frame_stats.iter().map(|stats| stats.chunks_unloaded);
        assert_eq!(loads.collect::<Vec<_>>(), [1, 6, 9]);
        assert_eq!(unloads.collect::<Vec<_>>(), [0, 0, 7]);
        assert!(frame_stats.iter().all(|stats| stats.bytes_read > 0));
        let first_frame = state.gpu_frame_stats()[0];
        assert!(
            first_frame.instances_drawn > 0,
            "the watched Chunk is drawn"
        );

        let counters = crate::bench::Counters::of(&frame_stats);
        assert_eq!(counters[crate::bench::Counter::ChunksLoaded], 16);
        assert_eq!(counters[crate::bench::Counter::ChunksUnloaded], 7);
        assert_eq!(counters[crate::bench::Counter::LateChunks], 2);
        let bytes_read: u64 = frame_stats.iter().map(|stats| stats.bytes_read).sum();
        assert_eq!(counters[crate::bench::Counter::BytesRead], bytes_read);
    }

    #[cfg(feature = "cook")]
    const FAR_FIELD_SIZE: (u32, u32) = (320, 180);

    /// The lock a test holds while it holds States of `world`: each takes
    /// 3 GB on lavapipe, and CI's runner has 16 GB.
    #[cfg(feature = "cook")]
    fn one_world_test_at_a_time() -> std::sync::MutexGuard<'static, ()> {
        static WORLD_STATES: std::sync::Mutex<()> = std::sync::Mutex::new(());
        WORLD_STATES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// A State of `world` moved by `shift`, and a target to read its frames
    /// back from.
    #[cfg(feature = "cook")]
    fn world_state(
        context: &crate::gpu::GpuContext,
        shift: glam::DVec3,
        switches: RenderSwitches,
    ) -> (State, wgpu::Texture) {
        let (width, height) = FAR_FIELD_SIZE;
        let format = crate::capture::TARGET_FORMAT;
        let scene = ReferenceScene::World;
        let state = State::new(context, format, width, height, scene, shift, switches)
            .expect("world loads");
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Far Field Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        (state, target)
    }

    /// Draws a frame from `camera`, Settled if `settle`, and reads it back.
    #[cfg(feature = "cook")]
    fn draw_from(
        context: &crate::gpu::GpuContext,
        state: &mut State,
        target: &wgpu::Texture,
        camera: crate::camera::Camera,
        settle: bool,
    ) -> Vec<u8> {
        state.simulation.world.insert_resource(camera);
        let frame_delta = ReferenceScene::World.frame_delta();
        if settle {
            state
                .update_settled(frame_delta)
                .expect("the frame settles");
        } else {
            state.update(frame_delta);
        }
        state.render(&target.create_view(&wgpu::TextureViewDescriptor::default()));
        crate::capture::read_back(context, target).expect("the frame reads back")
    }

    /// What draws the Chunk at `coordinates`.
    #[cfg(feature = "cook")]
    fn drawn_as(state: &State, coordinates: [i32; 2]) -> &'static str {
        let drawn = |chunks: &[crate::streaming::LoadedChunk]| {
            chunks
                .iter()
                .any(|chunk| chunk.coordinates == coordinates && !chunk.held)
        };
        let world = &state.simulation.world;
        let placements = drawn(&world.resource::<LoadedChunks>().0);
        let far_proxy = drawn(&world.resource::<ResidentFarProxies>().0);
        match (placements, far_proxy) {
            (true, false) => "its Placements",
            (false, true) => "its Far proxy",
            (true, true) => "both",
            (false, false) => "neither",
        }
    }

    /// The pixels of `image` that are not the ground's green.
    #[cfg(feature = "cook")]
    fn built_pixels(image: &[u8]) -> usize {
        let built = |pixel: &&[u8; 4]| pixel[1] <= pixel[0].max(pixel[2]).saturating_add(16);
        image.as_chunks::<4>().0.iter().filter(built).count()
    }

    /// With a Load radius of 0 m and no shadow rays, `world`'s camera hangs
    /// 250 m over the middle of a Chunk of its settlement, then of the next
    /// Chunk north. Over the first, that Chunk draws its Placements and the
    /// next its Far proxy. On the frame the camera comes over the next, a
    /// Late Chunk, it still draws its Far proxy, and the first, now held,
    /// draws its own again, their roofs and walls over half the picture.
    /// Once Settled, the next Chunk draws its Placements: more Instances and
    /// triangles than its Far proxy, as much roof and wall, and the same
    /// picture in over two pixels of three: its windows and the birds differ.
    #[cfg(feature = "cook")]
    #[test]
    fn a_far_proxy_draws_exactly_while_its_chunk_does_not() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let _world_states = one_world_test_at_a_time();
        let scene = ReferenceScene::World;
        let switches = RenderSwitches {
            shadow_rays: false,
            wait_for_static_tlas: true,
            ..RenderSwitches::default()
        };
        let (mut state, target) = world_state(&context, glam::DVec3::ZERO, switches);
        state.streaming.set_load_radius(0.0);
        let chunk_edge_metres = crate::cooked::CHUNK_EDGE_METRES;
        let over_roofs = scene.camera_at(599, glam::DVec3::ZERO).position;
        let next = [over_roofs.x, over_roofs.z].map(|metres| (metres / chunk_edge_metres).floor());
        let [next, first] = [0.0, 1.0].map(|south| [next[0] as i32, (next[1] + south) as i32]);
        let over = |[x, z]: [i32; 2]| {
            let middle = |coordinate: i32| (f64::from(coordinate) + 0.5) * chunk_edge_metres;
            let position = glam::dvec3(middle(x), 250.0, middle(z));
            crate::camera::Camera::looking_at(position, position + glam::dvec3(0.0, -250.0, -1.0))
        };

        draw_from(&context, &mut state, &target, over(first), true);
        assert_eq!(drawn_as(&state, first), "its Placements");
        assert_eq!(drawn_as(&state, next), "its Far proxy");
        let [_, albedo, _] = scene.far_proxy_baked_ids(next);
        let residency = state.renderer.material_pool.texture_residency(albedo);
        let (_, finest_resident_mip) = residency.expect("the Far proxy's albedo is loaded");
        assert_eq!(finest_resident_mip, 0, "its atlas streamed in whole");

        let late = draw_from(&context, &mut state, &target, over(next), false);
        assert_eq!(state.frame_stats().late_chunks, 1);
        assert_eq!(drawn_as(&state, first), "its Far proxy");
        assert_eq!(drawn_as(&state, next), "its Far proxy");

        let settled = draw_from(&context, &mut state, &target, over(next), true);
        assert_eq!(drawn_as(&state, first), "its Far proxy");
        assert_eq!(drawn_as(&state, next), "its Placements");

        state.flush_gpu_frame_stats().expect("GPU stats read back");
        let [late_stats, settled_stats] = [1, 2].map(|frame| state.gpu_frame_stats()[frame]);
        assert!(
            late_stats.instances_drawn < settled_stats.instances_drawn,
            "{} Instances draw the Far proxies, {} the Chunk",
            late_stats.instances_drawn,
            settled_stats.instances_drawn
        );
        assert!(late_stats.triangles_drawn < settled_stats.triangles_drawn);
        let difference = crate::capture::compare_images(&settled, &late);
        let pixels = (FAR_FIELD_SIZE.0 * FAR_FIELD_SIZE.1) as usize;
        let [late_built, settled_built] = [&late, &settled].map(|image| built_pixels(image));
        assert!(
            late_built * 2 > pixels && late_built.abs_diff(settled_built) * 20 < pixels,
            "the Far proxies cover {late_built} of {pixels} pixels, the Chunk {settled_built}"
        );
        assert!(
            difference.differing_pixels * 3 < pixels,
            "the Far proxy differs from its Chunk in {} of {pixels} pixels",
            difference.differing_pixels
        );
    }

    /// With the sun 34° up in the south, `world`'s camera hangs 250 m over
    /// the border of two Chunks of its settlement, a metre into the northern
    /// one, in two States: one of a Load radius of 0 m, where the southern
    /// Chunk draws its Far proxy, and one of 32 m, where both draw their
    /// Placements. In all but a pixel in 250, the northern Chunk's
    /// Placements take the shadows the Far proxy casts across the street as
    /// they take the Chunk's, and the frame's other half, where the Far
    /// proxy is shadowed as its Chunk is, differs in under a third of its
    /// pixels. A metre into the southern Chunk the two swap in the first
    /// State, over frames that neither settle nor run a Tick: every static
    /// TLAS build bound brings its changes within ten frames, and on the
    /// frame that binds the swap's the northern half, now the Far proxy's,
    /// is as bright as its Chunk's within a twentieth, where an unshadowed
    /// one is a tenth brighter. Settled, it differs from its Chunk as the
    /// southern one did.
    #[cfg(feature = "cook")]
    #[test]
    fn a_far_proxy_casts_and_takes_its_chunks_shadows_and_a_swap_is_traced_within_ten_frames() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let _world_states = one_world_test_at_a_time();
        let scene = ReferenceScene::World;
        let switches = RenderSwitches {
            wait_for_static_tlas: true,
            ..RenderSwitches::default()
        };
        let chunk_edge_metres = crate::cooked::CHUNK_EDGE_METRES;
        let over_roofs = scene.camera_at(599, glam::DVec3::ZERO).position;
        let [column, row] =
            [over_roofs.x, over_roofs.z].map(|metres| (metres / chunk_edge_metres).floor() as i32);
        let (north, south) = ([column, row], [column, row + 1]);
        let over_the_border = |metres_south: f64| {
            let middle_x = (f64::from(north[0]) + 0.5) * chunk_edge_metres;
            let border_z = f64::from(south[1]) * chunk_edge_metres;
            let position = glam::dvec3(middle_x, 250.0, border_z + metres_south);
            crate::camera::Camera::looking_at(position, position + glam::dvec3(0.0, -250.0, -1.0))
        };
        let low_sun = glam::vec3(0.2, -0.7, -1.0).normalize();
        let with_load_radius = |metres: f64| {
            let (mut state, target) = world_state(&context, glam::DVec3::ZERO, switches);
            state.streaming.set_load_radius(metres);
            let world = &mut state.simulation.world;
            let mut suns =
                world.query_filtered::<&mut Transform, With<crate::components::DirectionalLight>>();
            suns.single_mut(world).expect("one sun").rotation =
                glam::Quat::from_rotation_arc(glam::Vec3::NEG_Z, low_sun);
            (state, target)
        };
        let (mut swapping, swapping_target) = with_load_radius(0.0);
        let (mut loaded, loaded_target) = with_load_radius(32.0);
        // A static TLAS build is bound the frame after the one that settles.
        let traced_frame = |state: &mut State, target: &wgpu::Texture, metres_south: f64| {
            draw_from(&context, state, target, over_the_border(metres_south), true);
            draw_from(&context, state, target, over_the_border(metres_south), true)
        };
        let (width, height) = FAR_FIELD_SIZE;
        let street_rows = 4;
        let halves = [
            0..height / 2 - street_rows,
            height / 2 + street_rows..height,
        ];
        let half_pixels = (width * halves[0].len() as u32) as usize;
        let assert_shadowed_alike = |with_far_proxy: &[u8], with_chunk: &[u8], far_proxy_half| {
            let differing = halves.clone().map(|rows| {
                let row_bytes = 4 * width as usize;
                let bytes = rows.start as usize * row_bytes..rows.end as usize * row_bytes;
                crate::capture::compare_images(&with_chunk[bytes.clone()], &with_far_proxy[bytes])
                    .differing_pixels
            });
            let placements_differing = differing[1 - far_proxy_half];
            assert!(
                placements_differing * 250 < half_pixels,
                "the Far proxy shadows the Placements unlike its Chunk in {placements_differing} of {half_pixels} pixels"
            );
            let far_proxy_differing = differing[far_proxy_half];
            assert!(
                far_proxy_differing * 3 < half_pixels,
                "the Far proxy differs from its Chunk in {far_proxy_differing} of {half_pixels} pixels"
            );
        };

        let with_far_proxy = traced_frame(&mut swapping, &swapping_target, -1.0);
        assert_eq!(drawn_as(&swapping, north), "its Placements");
        assert_eq!(drawn_as(&swapping, south), "its Far proxy");
        let with_chunk = traced_frame(&mut loaded, &loaded_target, -1.0);
        assert_eq!(drawn_as(&loaded, north), "its Placements");
        assert_eq!(drawn_as(&loaded, south), "its Placements");
        assert_shadowed_alike(&with_far_proxy, &with_chunk, 1);

        let view = swapping_target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut frames_until_traced = Vec::new();
        let (mut swapped, mut swap_traced) = (false, None);
        for _ in 0..20_000 {
            swapping
                .simulation
                .world
                .insert_resource(over_the_border(1.0));
            swapping.update(Duration::ZERO);
            swapping.render(&view);
            let bound_after = swapping.frame_stats().frames_until_traced;
            frames_until_traced.extend(bound_after);
            if swapped && bound_after.is_some() {
                swap_traced = crate::capture::read_back(&context, &swapping_target).ok();
                break;
            }
            swapped = drawn_as(&swapping, south) == "its Placements";
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(drawn_as(&swapping, north), "its Far proxy");
        let swap_traced = swap_traced.expect("the swap's build is bound and its frame reads back");
        assert!(
            frames_until_traced.iter().all(|&frames| frames <= 10),
            "{frames_until_traced:?} frames"
        );
        let with_chunk = traced_frame(&mut loaded, &loaded_target, 1.0);
        assert_eq!(drawn_as(&loaded, north), "its Placements");
        let northern_brightness = |image: &[u8]| {
            let bytes = &image[..4 * (width * halves[0].end) as usize];
            bytes.iter().map(|&byte| u64::from(byte)).sum::<u64>()
        };
        let [far_proxy_brightness, chunk_brightness] =
            [&swap_traced, &with_chunk].map(|image| northern_brightness(image));
        assert!(
            far_proxy_brightness.abs_diff(chunk_brightness) * 20 < chunk_brightness,
            "the Far proxy's half sums to {far_proxy_brightness}, its Chunk's to {chunk_brightness}"
        );
        let with_far_proxy = traced_frame(&mut swapping, &swapping_target, 1.0);
        assert_shadowed_alike(&with_far_proxy, &with_chunk, 0);
    }

    /// `world`'s Far field and the ground past it, seen from its camera's
    /// frame 59, draw the same image with the world at the origin and moved
    /// by its own half size along both axes, whole Anchor cells: no World
    /// position reaches the GPU in f32 (ADR 0014).
    #[cfg(feature = "cook")]
    #[test]
    fn the_far_field_draws_alike_at_the_origin_and_at_the_worlds_far_corner() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let _world_states = one_world_test_at_a_time();
        let scene = ReferenceScene::World;
        let switches = RenderSwitches {
            wait_for_static_tlas: true,
            ..RenderSwitches::default()
        };
        let far_corner = glam::dvec3(5120.0, 0.0, 5120.0);
        let [at_origin, at_far_corner] = [glam::DVec3::ZERO, far_corner].map(|shift| {
            let (mut state, target) = world_state(&context, shift, switches);
            let image = draw_from(
                &context,
                &mut state,
                &target,
                scene.camera_at(59, shift),
                true,
            );
            let camera = scene.camera_at(59, shift);
            let far_proxies = &state.simulation.world.resource::<ResidentFarProxies>().0;
            let ahead_at_the_view_distance = far_proxies.iter().any(|proxy| {
                let to_proxy = proxy.placements[0].transform.translation - camera.position;
                let ahead = to_proxy.normalize().dot(camera.forward().as_dvec3()) > 0.95;
                !proxy.held && ahead && to_proxy.length() > scene.view_distance_metres()
            });
            assert!(
                ahead_at_the_view_distance,
                "a Far proxy 2 km out is in view"
            );
            image
        });
        assert!(
            at_origin == at_far_corner,
            "the far corner draws otherwise: {:?}",
            crate::capture::compare_images(&at_origin, &at_far_corner)
        );
    }

    /// Read before any loads, `texture_wall`'s four Chunks, each writing
    /// about 2 KB of its own Materials into the load staging, arrive one a
    /// frame under a 3,000-byte cap, nearest the camera first.
    #[test]
    fn streamed_loads_past_the_frames_load_bytes_wait_in_priority_order() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        const LOAD_BYTES_PER_FRAME: u64 = 3000;
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            64,
            36,
            ReferenceScene::TextureWall,
            glam::DVec3::ZERO,
            RenderSwitches::default(),
        )
        .expect("texture_wall loads");
        state
            .streaming
            .set_load_bytes_per_frame(LOAD_BYTES_PER_FRAME);
        let position = glam::dvec3(10.0, 5.0, 30.0);
        state
            .simulation
            .world
            .insert_resource(crate::camera::Camera::looking_at(
                position,
                position - glam::DVec3::Z,
            ));

        let mut written_bytes = Vec::new();
        for frame in 0..6 {
            let streamed = state
                .streaming
                .update(&mut state.simulation.world, &mut state.renderer, false)
                .expect("the Chunks stream");
            written_bytes.push(streamed.written_bytes);
            if frame == 0 {
                state.streaming.wait_for_reads().expect("the Chunks read");
            }
            upload_loads(&mut state);
        }
        for _ in 0..20_000 {
            if state.simulation.world.resource::<LoadedChunks>().0.len() == 4 {
                break;
            }
            state
                .streaming
                .update(&mut state.simulation.world, &mut state.renderer, false)
                .expect("the Chunks stream");
            upload_loads(&mut state);
            std::thread::sleep(Duration::from_millis(1));
        }

        let loaded = state.simulation.world.resource::<LoadedChunks>();
        let arrivals: Vec<[i32; 2]> = loaded.0.iter().map(|chunk| chunk.coordinates).collect();
        assert_eq!(arrivals, [[0, 0], [-1, 0], [0, -1], [-1, -1]]);
        assert_eq!(written_bytes[0], 0, "nothing read before the requests");
        let loading_frames = &written_bytes[1..5];
        assert!(
            loading_frames
                .iter()
                .all(|&bytes| bytes > 0 && bytes <= LOAD_BYTES_PER_FRAME),
            "{written_bytes:?}"
        );
        assert_eq!(written_bytes[5], 0);
    }

    /// What an upload does for the loads staged: submits them and takes
    /// their BLASes as far as the GPU has got.
    fn upload_loads(state: &mut State) {
        let renderer = &mut state.renderer;
        renderer.load_staging.submit(&renderer.queue);
        renderer.mesh_pool.compact_blases(&renderer.queue);
    }

    /// With a frame's Chunk loads capped at one Instance, each of
    /// `texture_wall`'s four Chunks places more alone, so each is placed in
    /// a frame of its own once its BLAS is ready, in the order loaded,
    /// nearest first, rather than all four at once.
    #[test]
    fn streamed_loads_past_the_frames_instances_wait_in_priority_order() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            64,
            36,
            ReferenceScene::TextureWall,
            glam::DVec3::ZERO,
            RenderSwitches::default(),
        )
        .expect("texture_wall loads");
        state.streaming.set_instances_placed_per_frame(1);
        let position = glam::dvec3(10.0, 5.0, 30.0);
        state
            .simulation
            .world
            .insert_resource(crate::camera::Camera::looking_at(
                position,
                position - glam::DVec3::Z,
            ));

        let mut chunks_loaded = Vec::new();
        for frame in 0..20_000 {
            let streamed = state
                .streaming
                .update(&mut state.simulation.world, &mut state.renderer, false)
                .expect("the Chunks stream");
            chunks_loaded.push(streamed.chunks_loaded);
            if frame == 0 {
                state.streaming.wait_for_reads().expect("the Chunks read");
            }
            if state.simulation.world.resource::<LoadedChunks>().0.len() == 4 {
                break;
            }
            upload_loads(&mut state);
            std::thread::sleep(Duration::from_millis(1));
        }

        let loaded = state.simulation.world.resource::<LoadedChunks>();
        let arrivals: Vec<[i32; 2]> = loaded.0.iter().map(|chunk| chunk.coordinates).collect();
        assert_eq!(arrivals, [[0, 0], [-1, 0], [0, -1], [-1, -1]]);
        assert_eq!(chunks_loaded[0], 0);
        chunks_loaded.retain(|&loaded| loaded > 0);
        assert_eq!(chunks_loaded, [1, 1, 1, 1]);
    }

    /// `shadows`' Chunk, streamed into frames that do not settle, brings a
    /// Mesh of its own: the frame that loads it places nothing and waits
    /// for no BLAS, nor does a later one, which places it once its BLAS is
    /// built and compacted, and its shadow is traced within ten frames of
    /// that one, which drew it unshadowed.
    #[test]
    fn a_streamed_chunk_is_placed_once_its_blas_is_ready_and_shadowed_within_ten_frames() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::Shadows;
        let (width, height) = (320, 180);
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            width,
            height,
            scene,
            glam::DVec3::ZERO,
            RenderSwitches {
                wait_for_static_tlas: true,
                ..RenderSwitches::default()
            },
        )
        .expect("shadows loads");
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Streamed Shadow Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::capture::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let brightness = |pixels: Vec<u8>| pixels.iter().map(|&byte| u64::from(byte)).sum::<u64>();

        let mut loaded_on = None;
        let mut placed = None;
        let mut traced = None;
        let mut waited_for_a_blas = false;
        for frame in 0..20_000_u32 {
            let camera = scene.camera_at(0, glam::DVec3::ZERO);
            state.simulation.world.insert_resource(camera);
            state.update(scene.frame_delta());
            state.render(&target_view);
            let stats = state.frame_stats();
            if stats.bytes_read > 0 {
                loaded_on.get_or_insert(frame);
            }
            let compacted =
                (stats.cpu_times.iter()).any(|(stage, _)| stage == crate::render::BLAS_COMPACTION);
            waited_for_a_blas |= loaded_on.is_some() && compacted;
            let read_back = || {
                let pixels = crate::capture::read_back(&context, &target);
                brightness(pixels.expect("the frame reads back"))
            };
            if stats.chunks_loaded > 0 {
                placed = Some((frame, read_back()));
            }
            if let (Some(_), Some(frames)) = (placed, stats.frames_until_traced) {
                traced = Some((frames, read_back()));
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }

        let loaded_on = loaded_on.expect("the Chunk loads");
        let (placed_on, unshadowed) = placed.expect("the Chunk is placed");
        let (frames_until_traced, shadowed) = traced.expect("the Chunk is traced");
        assert!(
            loaded_on < placed_on,
            "loaded on frame {loaded_on}, placed on {placed_on}"
        );
        assert!(!waited_for_a_blas);
        assert!(frames_until_traced <= 10, "{frames_until_traced} frames");
        assert!(
            shadowed < unshadowed,
            "{shadowed} of {unshadowed} with the shadow"
        );
    }

    /// The camera leaves `texture_wall` after its Chunks load into the pools
    /// and before any upload could have their Mesh's BLAS ready: none is
    /// ever placed, and each gives back the user it counted of that Mesh.
    #[test]
    fn a_chunk_the_camera_left_before_its_blas_was_ready_is_not_placed_and_frees_its_users() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        use crate::assets::{AssetReader as _, MeshUse};
        let scene = ReferenceScene::TextureWall;
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            64,
            36,
            scene,
            glam::DVec3::ZERO,
            RenderSwitches::default(),
        )
        .expect("texture_wall loads");
        let chunk = (state.streaming.reader().chunk(scene.chunk_id([0, 0]))).expect("a Chunk");
        let mesh = chunk.placements[0].mesh;
        let stream_from = |state: &mut State, position: glam::DVec3| {
            let camera = crate::camera::Camera::looking_at(position, position - glam::DVec3::Z);
            state.simulation.world.insert_resource(camera);
            let streamed = state
                .streaming
                .update(&mut state.simulation.world, &mut state.renderer, false)
                .expect("the Chunks stream");
            assert_eq!(streamed.chunks_loaded, 0);
            state
                .renderer
                .mesh_pool
                .users(mesh, MeshUse::DrawnAndTraced)
        };

        let beside = glam::dvec3(10.0, 5.0, 30.0);
        assert_eq!(stream_from(&mut state, beside), 0);
        state.streaming.wait_for_reads().expect("the Chunks read");
        assert!(stream_from(&mut state, beside) > 0);

        let far = beside + glam::dvec3(10_000.0, 0.0, 0.0);
        let mut users = u32::MAX;
        for _ in 0..20_000 {
            upload_loads(&mut state);
            users = stream_from(&mut state, far);
            if users == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(users, 0);
        assert!(
            state
                .simulation
                .world
                .resource::<LoadedChunks>()
                .0
                .is_empty()
        );
    }

    /// A Chunk whose 10 m quad samples a 2,048² Texture with mip 0 red, mip 1
    /// green and its Resident tail, mips 3 on, yellow, the camera teleported
    /// to each distance: seen from 200 m, only the tail loads and draws;
    /// 40 m away, mip 1 streams in, the finest the
    /// quad needs there; 0.2 m away, the next frame still draws mip 1, with
    /// one mip late, and once Settled draws mip 0, with none late. A second
    /// Placement of the quad, 800 m behind every camera position, only rays
    /// can reach, and its distance needs only mip 5: its own Texture keeps
    /// its Resident tail, the floor rays sample, throughout.
    #[cfg(feature = "cook")]
    #[test]
    fn a_textures_finer_mips_stream_in_finest_needed_as_the_camera_nears() {
        use crate::cooked::{
            AssetId, CookedCache, CookedChunk, CookedMaterial, CookedMesh, CookedMeshlet,
            CookedPlacement, CookedPrimitive, CookedTexture, CookedVertex, level_bytes,
        };

        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::Shadows;
        let directory =
            std::env::temp_dir().join(format!("praxis-mip-streaming-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let (texture_id, material_id, mesh_id) = (AssetId(1), AssetId(2), AssetId(3));
        let (behind_texture_id, behind_material_id) = (AssetId(4), AssetId(5));
        let format = wgpu::TextureFormat::Bc5RgUnorm;
        // Two BC4 blocks whose every texel takes the first endpoint.
        let block =
            |[red, green]: [u8; 2]| [red, red, 0, 0, 0, 0, 0, 0, green, green, 0, 0, 0, 0, 0, 0];
        let colours = [[255, 0], [0, 255], [0, 0]];
        let texture = CookedTexture {
            format,
            width: 2048,
            height: 2048,
            first_level: 0,
            levels: (0..12)
                .map(|level| {
                    let colour = colours.get(level).copied().unwrap_or([255, 255]);
                    block(colour).repeat(level_bytes(format, 2048, 2048, level) / 16)
                })
                .collect(),
        };
        let exposure = scene.camera_at(0, glam::DVec3::ZERO).exposure();
        let material = CookedMaterial {
            base_color: [0.0, 0.0, 0.0, 1.0],
            metallic: 1.0,
            roughness: 1.0,
            emissive: [1.0 / exposure; 3],
            normal_scale: 1.0,
            textures: [None, None, None, None, Some(texture_id)],
        };
        let behind_material = CookedMaterial {
            textures: [None, None, None, None, Some(behind_texture_id)],
            ..material
        };
        let half_size = 5.0;
        let vertices = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .map(|[u, v]: [f32; 2]| CookedVertex {
                position: [
                    (u - 0.5) * 2.0 * half_size,
                    (v - 0.5) * 2.0 * half_size,
                    0.0,
                ],
                uv: [u, v],
                qtangent: [0, 0, 0, i16::MAX],
            })
            .to_vec();
        let mesh = CookedMesh {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 6,
                material: Some(material_id),
                texel_density: 1.0 / (2.0 * half_size),
                thin_radius_metres: None,
                meshlets: vec![CookedMeshlet {
                    vertices: vec![0, 1, 2, 3],
                    triangles: vec![[0, 1, 2], [0, 2, 3]],
                }],
            }],
            coarser_lods: Vec::new(),
            bounds: [[-half_size, -half_size, 0.0], [half_size, half_size, 0.0]],
        };
        let chunk = CookedChunk {
            placements: vec![
                CookedPlacement {
                    mesh: mesh_id,
                    material: None,
                    translation: glam::DVec3::ZERO,
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                CookedPlacement {
                    mesh: mesh_id,
                    material: Some(behind_material_id),
                    translation: glam::dvec3(0.0, 0.0, 800.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
            ],
            lights: Vec::new(),
            bounds: [glam::dvec3(-5.0, -5.0, 0.0), glam::dvec3(5.0, 5.0, 800.0)],
        };
        let texture_blob = texture.to_blob(1).expect("a BC5 Blob");
        let blobs = [
            (texture_id, texture_blob.clone()),
            (behind_texture_id, texture_blob),
            (material_id, material.to_blob()),
            (behind_material_id, behind_material.to_blob()),
            (mesh_id, mesh.to_blob().expect("a Mesh Blob")),
            (
                scene.chunk_id([0, 0]),
                chunk.to_blob().expect("a Chunk Blob"),
            ),
        ];
        for (id, blob) in blobs {
            cache.write(id, "key", &blob).expect("Blob written");
        }

        let (width, height) = (128, 72);
        let mut state = State::new(
            &context,
            crate::capture::TARGET_FORMAT,
            width,
            height,
            scene,
            glam::DVec3::ZERO,
            RenderSwitches::default(),
        )
        .expect("shadows loads");
        state.streaming = ChunkStreaming::new(
            scene,
            glam::DVec3::ZERO,
            scene.load_radius_metres(),
            state.renderer.capacities().load_bytes_per_frame,
            Arc::new(cache),
        )
        .expect("the IO pool starts");
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mip Streaming Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::capture::TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut draw = |distance: f64, settle: bool| {
            let mut camera = crate::camera::Camera::looking_at(
                glam::dvec3(0.0, 0.0, distance),
                glam::DVec3::ZERO,
            );
            camera.ev100 = scene.camera_at(0, glam::DVec3::ZERO).ev100;
            camera.teleported = true;
            state.simulation.world.insert_resource(camera);
            if settle {
                state
                    .update_settled(scene.frame_delta())
                    .expect("the frame settles");
            } else {
                state.update(scene.frame_delta());
            }
            state.render(&target_view);
            let pixels =
                crate::capture::read_back(&context, &target).expect("the frame reads back");
            let centre = ((height / 2 * width + width / 2) * 4) as usize;
            let [red, green] = [pixels[centre], pixels[centre + 1]];
            let residency = state.renderer.material_pool.texture_residency(texture_id);
            let resident_level = residency.map(|(_, level)| level);
            let behind_residency = state
                .renderer
                .material_pool
                .texture_residency(behind_texture_id);
            let behind_resident_level = behind_residency.map(|(_, level)| level);
            (
                red,
                green,
                resident_level,
                state.frame_stats().late_mips,
                behind_resident_level,
            )
        };
        let far = draw(200.0, true);
        let middle = draw(40.0, true);
        let near = draw(0.2, false);
        let near_settled = draw(0.2, true);
        std::fs::remove_dir_all(&directory).ok();

        let yellow = |red: u8, green: u8| red > 128 && green > 128;
        let green = |red: u8, green: u8| green > 128 && red < green / 4;
        let red = |red: u8, green: u8| red > 128 && green < red / 4;
        assert!(
            yellow(far.0, far.1) && far.2 == Some(3) && far.3 == 0,
            "200 m: {far:?}"
        );
        assert_eq!((middle.2, middle.3), (Some(1), 0), "40 m: {middle:?}");
        assert!(
            green(near.0, near.1) && near.2 == Some(1) && near.3 == 1,
            "0.2 m: {near:?}"
        );
        assert!(
            red(near_settled.0, near_settled.1) && near_settled.2 == Some(0) && near_settled.3 == 0,
            "0.2 m, Settled: {near_settled:?}"
        );
        for drawn in [far, middle, near, near_settled] {
            assert_eq!(drawn.4, Some(3), "behind the camera: {drawn:?}");
        }
    }
}
