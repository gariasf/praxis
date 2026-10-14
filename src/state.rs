use std::time::{Duration, Instant};

use anyhow::Context;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

use crate::assets::{MaterialData, MaterialPool, MeshPool};
use crate::camera::fly_camera;
use crate::components::{MaterialRef, MeshRef, PointLight, Transform};
use crate::gpu::GpuContext;
use crate::helmet::{HelmetAssets, RuntimeHelmets, despawn_helmet, spawn_helmet};
use crate::input::{Input, clear_just_pressed};
use crate::render::{FrameData, InstanceSlots, PointLightQuery, Renderer, prepare_frame};
use crate::scenes::{
    ReferenceScene, SWEEP_METALLIC, SWEEP_ROUGHNESS, insert_gltf_model, spawn_far_coplanar,
    spawn_normal_tangent, spawn_orrery, spawn_pbr_sweep, spawn_uv_grid, spin,
};
use crate::stats::{FrameRecorder, FrameStats};
use crate::threading::{init_compute_pool, single_threaded_schedule};
use crate::time::{FixedTime, Time, fixed_schedule, run_ticks};
use crate::transform::{
    propagate_world_transforms, record_previous_world_transforms, reset_teleported,
};

/// The helmet-class model the Scene draws, relative to the working directory
/// (the crate root under `cargo run` and `cargo test`). It is fetched, not
/// committed: `scripts/fetch-assets.sh` downloads it.
pub const DEFAULT_MODEL: &str = "assets/SciFiHelmet/SciFiHelmet.gltf";

/// The Scene in its ECS World, the Frame data prepared from it, and the
/// Renderer that draws that into a target view.
pub struct State {
    // World is an internal detail, it should not be public. But this will do for now.
    pub world: World,
    schedule: Schedule,
    fixed_schedule: Schedule,
    recorder: FrameRecorder,
    frame: FrameData,
    renderer: Renderer,
    point_light_query: PointLightQuery,
    instance_slots: InstanceSlots,
}

impl State {
    /// A State that renders `scene`, moved with its camera by `shift` metres,
    /// into `width` × `height` views of `target_format` textures, both
    /// non-zero, seen from the scene's camera at frame 0.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scene: ReferenceScene,
        shift: glam::DVec3,
    ) -> anyhow::Result<State> {
        let _span = tracing::info_span!("state_init").entered();
        init_compute_pool();

        let mut renderer = Renderer::new(context, target_format, width, height);
        let Renderer {
            device,
            queue,
            mesh_pool,
            material_pool,
            ..
        } = &mut renderer;

        let mut world = World::new();
        world.insert_resource(Input::default());
        world.insert_resource(Time::default());
        world.insert_resource(FixedTime::default());
        world.insert_resource(scene.camera_at(0, shift));
        world.insert_resource(RuntimeHelmets::default());

        let recorder = FrameRecorder::new();
        let (schedule, mut fixed_schedule) = simulation_schedules(&recorder);

        match scene {
            ReferenceScene::Helmet => {
                Self::spawn_helmet_scene(&mut world, device, queue, mesh_pool, material_pool)?
            }
            ReferenceScene::PbrSweep => {
                spawn_pbr_sweep(&mut world, device, queue, mesh_pool, material_pool)
            }
            ReferenceScene::UvGrid => {
                spawn_uv_grid(&mut world, device, queue, mesh_pool, material_pool)
            }
            ReferenceScene::NormalTangent => {
                spawn_normal_tangent(&mut world, device, queue, mesh_pool, material_pool)?
            }
            ReferenceScene::FarCoplanar => {
                spawn_far_coplanar(&mut world, device, queue, mesh_pool, material_pool)
            }
            ReferenceScene::Orrery => {
                spawn_orrery(&mut world, device, queue, mesh_pool, material_pool)
            }
        }
        for (position, color) in [
            (glam::dvec3(2.0, 2.0, 2.0), glam::Vec3::X),
            (glam::dvec3(-2.0, 2.0, 2.0), glam::Vec3::Y),
            (glam::dvec3(2.0, 2.0, -2.0), glam::Vec3::Z),
            (glam::dvec3(-2.0, 2.0, -2.0), glam::vec3(1.0, 1.0, 0.0)),
        ] {
            world.spawn((
                Transform::from_translation(position),
                PointLight {
                    color,
                    intensity: 1.0,
                },
            ));
        }
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
        let point_light_query = world.query();

        Ok(Self {
            world,
            schedule,
            fixed_schedule,
            recorder,
            frame: FrameData::default(),
            renderer,
            point_light_query,
            instance_slots: InstanceSlots::default(),
        })
    }

    /// Spawns `DEFAULT_MODEL` and a factor sweep of helmets
    /// around it, and records the helmet's Handles for the spawn systems.
    fn spawn_helmet_scene(
        world: &mut World,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_pool: &mut MeshPool,
        material_pool: &mut MaterialPool,
    ) -> anyhow::Result<()> {
        let (mesh_handle, material_handle) =
            insert_gltf_model(device, queue, mesh_pool, material_pool, DEFAULT_MODEL)
                .context("run scripts/fetch-assets.sh if the model is missing")?;

        // Debug material sweep: a pure factor sweep over the neutral default
        // layers with a white base color. Top row dielectric, bottom row metal;
        // roughness grows left to right. The textured glTF helmet sits between
        // the rows. Placed 8 units ahead of the default camera. SciFiHelmet is
        // 2.9 units tall, so the sweep copies are scaled to 3/4 to clear the
        // middle helmet and stay in frame.
        for (row_index, metallic) in SWEEP_METALLIC.into_iter().enumerate() {
            for (column_index, roughness) in SWEEP_ROUGHNESS.into_iter().enumerate() {
                let variant_handle = material_pool.insert(
                    queue,
                    MaterialData {
                        base_color: [1.0, 1.0, 1.0, 1.0],
                        metallic_roughness: [metallic, roughness, 0.0, 0.0],
                        emissive: [0.0, 0.0, 0.0, 1.0],
                        texture_indices: [MaterialPool::DEFAULT_LAYER; 4],
                        extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                    },
                );
                let column_offset = (column_index as f32 - 2.0) * 2.0;
                let row_offset = 2.6 - row_index as f32 * 5.2;
                world.spawn((
                    Transform {
                        scale: glam::Vec3::splat(0.75),
                        ..Transform::from_translation(glam::dvec3(
                            column_offset.into(),
                            row_offset.into(),
                            -8.0,
                        ))
                    },
                    MeshRef(mesh_handle),
                    MaterialRef(variant_handle),
                ));
            }
        }
        world.spawn((
            Transform::from_translation(glam::dvec3(0.0, 0.0, -8.0)),
            MeshRef(mesh_handle),
            MaterialRef(material_handle),
        ));

        world.insert_resource(HelmetAssets {
            mesh: mesh_handle,
            material: material_handle,
        });
        Ok(())
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    /// Runs the Ticks `frame_delta` completes, moves the camera by it,
    /// prepares the frame's Frame data, blended between the two newest Ticks,
    /// and uploads it.
    pub fn update(&mut self, frame_delta: Duration) {
        self.recorder.begin();
        self.recorder.dropped_time =
            run_ticks(&mut self.fixed_schedule, &mut self.world, frame_delta);
        self.world.resource_mut::<Time>().delta_time = frame_delta.as_secs_f32();
        self.schedule.run(&mut self.world);

        let started = Instant::now();
        // Writes a failed upload lost are written again, with every Instance.
        if self.renderer.take_lost_instance_writes() {
            self.instance_slots = InstanceSlots::default();
        }
        prepare_frame(
            &mut self.world,
            &mut self.point_light_query,
            &mut self.instance_slots,
            &self.renderer.mesh_pool,
            self.renderer.aspect_ratio(),
            &mut self.frame,
        );
        self.recorder.stage("prepare_frame", started);
        // Ends the window in which prepare sees changes and removals, and
        // drains the removals, which otherwise pile up.
        self.world.clear_trackers();

        self.renderer.upload(&self.frame, &mut self.recorder);
    }

    /// Draws the frame into `target`, a view in the format and size this
    /// State renders to, and times its one pass into `timestamp_writes`.
    /// Returns the frame's submission.
    pub fn render(
        &mut self,
        target: &wgpu::TextureView,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) -> wgpu::SubmissionIndex {
        let submission =
            self.renderer
                .render(&self.frame, target, timestamp_writes, &mut self.recorder);
        self.recorder.end();
        submission
    }

    /// The stats of the last frame, from `update` through `render`.
    pub fn frame_stats(&self) -> FrameStats {
        FrameStats {
            gpu_memory_bytes: self.renderer.gpu_memory_bytes(),
            allocator_report_bytes: self.renderer.allocator_report_bytes(),
            ..self.recorder.stats()
        }
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
        // A Tick starts by keeping the World transforms it may move and ends
        // by propagating its moves, so the two newest Ticks' are at hand for
        // prepare's blend, however many Ticks a frame runs (ADR 0022).
        // Game-state systems run before `reset_teleported`, which reads the
        // jumps they mark. Presses wait in `Input` for the next Tick that
        // runs, so a frame that runs none loses none.
        fixed_schedule.add_systems(
            (
                record_previous_world_transforms,
                (spawn_helmet, despawn_helmet)
                    .chain()
                    .run_if(resource_exists::<HelmetAssets>),
                spin,
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
}
