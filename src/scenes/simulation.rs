use std::time::Duration;

// Shadows the prelude's `ContextExt`, whose `context` and `with_context` make anyhow's ambiguous.
use anyhow::Context as ContextExt;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

use super::{ReferenceScene, WrapsAroundCamera, spin, sway, wrap_around_camera};
use crate::assets::AssetLoader;
use crate::camera::{Camera, CameraPath, fly_camera};
use crate::components::Transform;
use crate::helmet::{HelmetAssets, RuntimeHelmets, despawn_helmet, spawn_helmet};
use crate::input::{Input, clear_just_pressed};
use crate::render::CrowdForm;
use crate::stats::FrameRecorder;
use crate::streaming::{FarGround, LoadedChunks, ResidentFarProxies};
use crate::threading::single_threaded_schedule;
use crate::time::{FixedTime, Time, fixed_schedule, run_ticks};
use crate::transform::{
    propagate_world_transforms, record_previous_curvatures, record_previous_world_transforms,
    reset_teleported,
};

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

        scene
            .spawn(&mut world, loader, crowd_form)
            .with_context(|| format!("loading {}", scene.name()))?;
        world.insert_resource(LoadedChunks::default());
        world.insert_resource(ResidentFarProxies::default());
        world.insert_resource(scene.environment());
        world.spawn(scene.sun());
        // A child's translation is relative to its parent, so only roots move.
        let mut root_query = world.query_filtered::<&mut Transform, Without<ChildOf>>();
        for mut transform in root_query.iter_mut(&mut world) {
            transform.translation += shift;
        }
        if let Some(mut far_ground) = world.get_resource_mut::<FarGround>()
            && let Some(far_ground) = &mut far_ground.0
        {
            far_ground.shift(shift);
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

    /// Moves the camera by `frame_delta`, runs the Ticks it completes, each
    /// sampling the camera's move where the Tick ends, and returns the time
    /// past the Ticks' cap it dropped. A State's next `prepare` writes every
    /// change since its last.
    pub(crate) fn simulate(&mut self, frame_delta: Duration) -> Duration {
        let start = self.world.resource::<Camera>().position;
        self.world.resource_mut::<Time>().delta_time = frame_delta.as_secs_f32();
        self.schedule.run(&mut self.world);
        let end = self.world.resource::<Camera>().position;
        self.world.insert_resource(CameraPath { start, end });
        run_ticks(&mut self.fixed_schedule, &mut self.world, frame_delta)
    }
}

/// The schedule run once per frame and the fixed schedule run once per
/// Tick: their systems, timed by `recorder`, on the single-threaded executor.
pub(crate) fn simulation_schedules(recorder: &FrameRecorder) -> (Schedule, Schedule) {
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
                wrap_around_camera.run_if(any_with_component::<WrapsAroundCamera>),
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

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::transform::WorldTransform;

    #[derive(Resource, Default)]
    struct FlockTrail(Vec<Vec<glam::DAffine3>>);

    fn record_flocks(
        centres: Query<&WorldTransform, With<WrapsAroundCamera>>,
        mut trail: ResMut<FlockTrail>,
    ) {
        trail
            .0
            .push(centres.iter().map(|centre| centre.0).collect());
    }

    /// `world`'s flock centres' World transforms after each of the first
    /// `ticks` Ticks, flown through at 200 m/s with frames of `frame_deltas`
    /// in turn.
    fn flock_trail(ticks: usize, frame_deltas: &[Duration]) -> Vec<Vec<glam::DAffine3>> {
        let mut simulation = Simulation::spawned(
            ReferenceScene::World,
            glam::DVec3::ZERO,
            CrowdForm::default(),
            &mut crate::cook::AssetRecorder::default(),
            &FrameRecorder::new(),
        )
        .unwrap();
        let world = &mut simulation.world;
        world.insert_resource(FlockTrail::default());
        world.resource_mut::<Camera>().speed = 200.0;
        let forward = winit::keyboard::KeyCode::KeyW;
        world.resource_mut::<Input>().pressed.insert(forward);
        let recorded = record_flocks.after(propagate_world_transforms);
        simulation.fixed_schedule.add_systems(recorded);
        for frame_delta in frame_deltas.iter().cycle() {
            if simulation.world.resource::<FlockTrail>().0.len() >= ticks {
                break;
            }
            simulation.simulate(*frame_delta);
        }
        let mut trail = simulation.world.remove_resource::<FlockTrail>().unwrap().0;
        trail.truncate(ticks);
        trail
    }

    #[test]
    fn worlds_flocks_wrap_on_the_same_ticks_whatever_the_frame_timing() {
        const TICKS: usize = 192;
        let steady = flock_trail(TICKS, &[crate::time::SIXTY_HERTZ]);
        let mixed_rates = [6_944_444, 16_666_667, 33_333_333, 11_111_111].map(Duration::from_nanos);
        let mixed = flock_trail(TICKS, &mixed_rates);
        let translations = |tick: usize| -> Vec<_> {
            steady[tick]
                .iter()
                .map(|centre| centre.translation)
                .collect()
        };
        assert_ne!(translations(0), translations(TICKS - 1), "no flock jumped");
        let bits = |centres: &[glam::DAffine3]| -> Vec<_> {
            let centre_bits = |centre: &glam::DAffine3| centre.to_cols_array().map(f64::to_bits);
            centres.iter().map(centre_bits).collect()
        };
        for (tick, (steady, mixed)) in steady.iter().zip(&mixed).enumerate() {
            assert!(
                bits(steady) == bits(mixed),
                "flock centres differ after Tick {tick}"
            );
        }
    }
}
