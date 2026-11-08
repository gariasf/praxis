use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

use crate::assets::{AssetLoader, PoolLoader};
use crate::camera::fly_camera;
use crate::components::Transform;
use crate::cooked::{COOKED_CACHE_DIRECTORY, CookedCache};
use crate::gpu::GpuContext;
use crate::helmet::{HelmetAssets, RuntimeHelmets, despawn_helmet, spawn_helmet};
use crate::input::{Input, clear_just_pressed};
use crate::render::{CrowdForm, FramePreparer, RenderSwitches, Renderer};
use crate::scenes::{
    ReferenceScene, sci_fi_helmet_mesh_id, spawn_crowd, spawn_orrery, spawn_texture_wall,
    spawn_town, spawn_traced_crowd, spin, sway,
};
use crate::stats::{FrameRecorder, FrameStats, GpuFrameStats};
use crate::streaming::{ChunkStreaming, LoadedChunks};
use crate::threading::{init_compute_pool, single_threaded_schedule};
use crate::time::{FixedTime, Time, fixed_schedule, run_ticks};
use crate::transform::{
    propagate_world_transforms, record_previous_curvatures, record_previous_world_transforms,
    reset_teleported,
};

/// The source file of the helmet scene's Mesh and Material, relative to the
/// working directory (the crate root under `cargo run` and `cargo test`),
/// which a dev launch cooks. It is fetched, not committed: `praxis fetch`
/// downloads it, as a dev launch of the helmet does when it is absent.
pub const DEFAULT_MODEL: &str = "assets/SciFiHelmet/SciFiHelmet.gltf";

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

/// A Reference scene's ECS World and the schedules that advance it: what
/// prepare reads, with no Renderer.
pub(crate) struct Simulation {
    pub(crate) world: World,
    schedule: Schedule,
    fixed_schedule: Schedule,
}

impl Simulation {
    /// `scene`'s World at frame 0, as `spawned` leaves it, with every Chunk
    /// of the scene loaded through `loader` and moved by `shift` metres.
    #[cfg(feature = "cook")]
    pub(crate) fn new(
        scene: ReferenceScene,
        shift: glam::DVec3,
        crowd_form: CrowdForm,
        loader: &mut dyn AssetLoader,
        recorder: &FrameRecorder,
    ) -> anyhow::Result<Self> {
        let mut simulation = Self::spawned(scene, shift, crowd_form, loader, recorder)?;
        let mut chunks = scene
            .load_chunks(loader)
            .with_context(|| format!("loading {}'s Chunks", scene.name()))?;
        for chunk in &mut chunks.0 {
            chunk.shift(shift);
        }
        simulation.world.insert_resource(chunks);
        Ok(simulation)
    }

    /// `scene`'s World at frame 0, moved with its camera by `shift` metres,
    /// its crowd spawned for `crowd_form` and its spawn's Assets loaded
    /// through `loader`, with its systems timed by `recorder`, and no Chunk
    /// loaded.
    pub(crate) fn spawned(
        scene: ReferenceScene,
        shift: glam::DVec3,
        crowd_form: CrowdForm,
        loader: &mut dyn AssetLoader,
        recorder: &FrameRecorder,
    ) -> anyhow::Result<Self> {
        let mut world = World::new();
        world.insert_resource(Input::default());
        world.insert_resource(Time::default());
        world.insert_resource(FixedTime::default());
        world.insert_resource(scene.camera_at(0, shift));
        world.insert_resource(RuntimeHelmets::default());

        let (schedule, mut fixed_schedule) = simulation_schedules(recorder);

        let spawn: fn(&mut World, &mut dyn AssetLoader) -> anyhow::Result<()> = match scene {
            ReferenceScene::Helmet => State::spawn_helmet_scene,
            ReferenceScene::PbrSweep
            | ReferenceScene::UvGrid
            | ReferenceScene::NormalTangent
            | ReferenceScene::FarCoplanar
            | ReferenceScene::Minification
            | ReferenceScene::Shadows => |_, _| Ok(()),
            ReferenceScene::Orrery => spawn_orrery,
            ReferenceScene::TextureWall => spawn_texture_wall,
            ReferenceScene::Crowd => match crowd_form {
                CrowdForm::Proxies => spawn_crowd,
                CrowdForm::Refit | CrowdForm::Rebuild => spawn_traced_crowd,
            },
            ReferenceScene::Town => spawn_town,
        };
        spawn(&mut world, loader).with_context(|| format!("loading {}", scene.name()))?;
        world.insert_resource(LoadedChunks::default());
        world.insert_resource(scene.environment());
        world.spawn(scene.sun());
        // A child's translation is relative to its parent, so only roots move.
        let mut root_query = world.query_filtered::<&mut Transform, Without<ChildOf>>();
        for mut transform in root_query.iter_mut(&mut world) {
            transform.translation += shift;
        }
        // Ticks propagate World transforms as they end; frames drawn before
        // the first Tick draw these.
        world
            .run_system_once(propagate_world_transforms)
            .map_err(|error| anyhow::anyhow!("World transform propagation: {error}"))?;
        fixed_schedule
            .initialize(&mut world)
            .context("fixed schedule build")?;
        Ok(Self {
            world,
            schedule,
            fixed_schedule,
        })
    }

    /// Runs the Ticks `frame_delta` completes and moves the camera by it,
    /// and returns the time past the Ticks' cap it dropped. A State's next
    /// `prepare` writes every change since its last.
    pub(crate) fn simulate(&mut self, frame_delta: Duration) -> Duration {
        let dropped_time = run_ticks(&mut self.fixed_schedule, &mut self.world, frame_delta);
        self.world.resource_mut::<Time>().delta_time = frame_delta.as_secs_f32();
        self.schedule.run(&mut self.world);
        dropped_time
    }
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
        init_compute_pool();
        if let Some(reason) = switches.crowd_form.unavailable_on(context.backend()) {
            anyhow::bail!("{:?} is unavailable: {reason}", switches.crowd_form);
        }

        let mut renderer = Renderer::new(context, target_format, width, height, switches);
        let Renderer {
            device,
            mesh_pool,
            material_pool,
            load_staging,
            ..
        } = &mut renderer;

        let recorder = FrameRecorder::new();
        let cache = Arc::new(CookedCache::new(COOKED_CACHE_DIRECTORY));
        let mut streaming =
            ChunkStreaming::new(scene, shift, scene.load_radius_metres(), cache.clone())?;
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

    /// Records `DEFAULT_MODEL`'s helmet Mesh for the spawn systems; the
    /// scene's Chunks hold its helmets.
    fn spawn_helmet_scene(world: &mut World, loader: &mut dyn AssetLoader) -> anyhow::Result<()> {
        let mesh = loader.mesh(sci_fi_helmet_mesh_id()).with_context(|| {
            format!("loading {DEFAULT_MODEL}; run `praxis fetch` if it is missing")
        })?;
        world.insert_resource(HelmetAssets { mesh });
        Ok(())
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    /// Runs the Ticks `frame_delta` completes, moves the camera by it,
    /// streams the Chunks around it, prepares the frame's Frame data,
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
        self.recorder.counters.chunks_loaded = streamed.chunks_loaded;
        self.recorder.counters.chunks_unloaded = streamed.chunks_unloaded;
        self.recorder.counters.bytes_read = streamed.bytes_read;
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

    /// One past the last frame whose submission completed, as far as the
    /// submissions and polls since have seen.
    #[cfg(test)]
    pub(crate) fn frames_completed(&self) -> u64 {
        self.renderer.frames_completed()
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

/// The State's one schedule: its systems, timed by `recorder`, on the
/// single-threaded executor.
/// The schedule run once per frame and the fixed schedule run once per Tick.
fn simulation_schedules(recorder: &FrameRecorder) -> (Schedule, Schedule) {
    let mut schedule = single_threaded_schedule();
    let mut fixed_schedule = fixed_schedule();
    recorder.systems.scope(|| {
        schedule.add_systems(fly_camera);
        // A Tick starts by keeping the World transforms and curvatures it may
        // change and ends by propagating its moves, so the two newest Ticks'
        // are at hand for prepare's blend, however many Ticks a frame runs
        // (ADR 0022).
        // Game-state systems run before `reset_teleported`, which reads the
        // jumps they mark. Presses wait in `Input` for the next Tick that
        // runs, so a frame that runs none loses none.
        fixed_schedule.add_systems(
            (
                record_previous_world_transforms,
                record_previous_curvatures,
                (spawn_helmet, despawn_helmet)
                    .chain()
                    .run_if(resource_exists::<HelmetAssets>),
                spin,
                sway,
                clear_just_pressed,
                reset_teleported,
                propagate_world_transforms,
            )
                .chain(),
        );
    });
    (schedule, fixed_schedule)
}

#[cfg(test)]
mod tests {
    use bevy_ecs::schedule::ExecutorKind;

    use super::*;
    use crate::assets::MeshHandle;
    use crate::components::Bend;
    use crate::time::{TICK, TICK_SECONDS};
    use crate::transform::{PreviousWorldTransform, Teleported, WorldTransform};

    #[test]
    fn simulation_schedules_run_single_threaded() {
        let (schedule, fixed_schedule) = simulation_schedules(&FrameRecorder::new());
        assert_eq!(schedule.get_executor_kind(), ExecutorKind::SingleThreaded);
        assert_eq!(
            fixed_schedule.get_executor_kind(),
            ExecutorKind::SingleThreaded
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
        assert_eq!(loads.collect::<Vec<_>>(), [1, 8, 9]);
        assert_eq!(unloads.collect::<Vec<_>>(), [0, 0, 9]);
        assert!(frame_stats.iter().all(|stats| stats.bytes_read > 0));
        let first_frame = state.gpu_frame_stats()[0];
        assert!(
            first_frame.instances_drawn > 0,
            "the watched Chunk is drawn"
        );

        let counters = crate::bench::Counters::of(&frame_stats);
        assert_eq!(counters.chunks_loaded, 18);
        assert_eq!(counters.chunks_unloaded, 9);
        assert_eq!(counters.late_chunks, 2);
        let bytes_read: u64 = frame_stats.iter().map(|stats| stats.bytes_read).sum();
        assert_eq!(counters.bytes_read, bytes_read);
    }
}
