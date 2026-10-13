use std::time::{Duration, Instant};

use anyhow::Context;
use bevy_ecs::prelude::*;

use crate::assets::{MaterialData, MaterialPool, MeshPool};
use crate::camera::fly_camera;
use crate::components::{MaterialRef, MeshRef, PointLight, Transform};
use crate::gpu::GpuContext;
use crate::helmet::{HelmetAssets, RuntimeHelmets, despawn_helmet, spawn_helmet};
use crate::input::{Input, clear_just_pressed};
use crate::render::{FrameData, Renderer, prepare_frame};
use crate::scenes::{
    ReferenceScene, insert_gltf_model, spawn_far_coplanar, spawn_normal_tangent, spawn_pbr_sweep,
    spawn_uv_grid,
};
use crate::stats::{FrameRecorder, FrameStats};
use crate::threading::{init_compute_pool, single_threaded_schedule};
use crate::time::{FixedTime, Time, fixed_schedule, run_ticks};

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
    point_light_query: QueryState<(&'static Transform, &'static PointLight)>,
}

impl State {
    /// A State that renders `scene` into `width` × `height` views of
    /// `target_format` textures, both non-zero, seen from the scene's camera at
    /// frame 0.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scene: ReferenceScene,
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
        world.insert_resource(scene.camera_at(0));
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
        for (row_index, metallic) in [0.0_f32, 1.0].into_iter().enumerate() {
            for (column_index, roughness) in [0.1_f32, 0.3, 0.5, 0.7, 0.9].into_iter().enumerate() {
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

    /// Moves the camera by `frame_delta`, runs the Ticks it completes,
    /// prepares the frame's Frame data and uploads it.
    pub fn update(&mut self, frame_delta: Duration) {
        self.recorder.begin();
        self.world.resource_mut::<Time>().delta_time = frame_delta.as_secs_f32();
        self.schedule.run(&mut self.world);
        self.recorder.dropped_time =
            run_ticks(&mut self.fixed_schedule, &mut self.world, frame_delta);

        let started = Instant::now();
        prepare_frame(
            &mut self.world,
            &mut self.point_light_query,
            &self.renderer.mesh_pool,
            self.renderer.aspect_ratio(),
            &mut self.frame,
        );
        self.recorder.stage("prepare_frame", started);

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
            allocator_report_bytes: self
                .renderer
                .device
                .generate_allocator_report()
                .map(|report| report.total_allocated_bytes),
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
        // Presses wait in `Input` for the next Tick that runs, so a frame
        // that runs none loses none.
        fixed_schedule.add_systems(
            (
                (spawn_helmet, despawn_helmet)
                    .chain()
                    .run_if(resource_exists::<HelmetAssets>),
                clear_just_pressed,
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

    #[test]
    fn simulation_schedules_run_single_threaded() {
        let (schedule, fixed_schedule) = simulation_schedules(&FrameRecorder::new());
        assert_eq!(schedule.get_executor_kind(), ExecutorKind::SingleThreaded);
        assert_eq!(
            fixed_schedule.get_executor_kind(),
            ExecutorKind::SingleThreaded
        );
    }
}
