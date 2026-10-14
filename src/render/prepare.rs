use std::ops::Range;

use bevy_ecs::prelude::*;

use crate::assets::MeshPool;
use crate::camera::{Camera, projection_matrix};
use crate::components::{MaterialRef, MeshRef, PointLight};
use crate::render::instance::{anchor_and_offset, padded_columns};
use crate::render::{CameraUniform, InstanceData, LightUniform};
use crate::time::FixedTime;
use crate::transform::{PreviousWorldTransform, WorldTransform};

/// Everything the renderer draws in one frame, owned, so it holds no borrow
/// into the ECS World and can cross to another thread (ADRs 0004, 0016).
#[derive(Default)]
pub struct FrameData {
    pub camera: CameraUniform,
    pub light: LightUniform,
    /// Slots the Instance buffer holds: one past the highest any Renderable
    /// has taken.
    pub instance_slot_count: u32,
    /// The Instances this frame writes, in `instance_writes` order; every
    /// other slot keeps what an earlier frame wrote (ADR 0003).
    pub instances: Vec<InstanceData>,
    pub instance_writes: Vec<InstanceWrite>,
    pub draws: Vec<Draw>,
}

const _: () = {
    const fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<FrameData>()
};

/// The point lights, with what blends the moving ones. The caller keeps it
/// across frames, so preparing allocates nothing.
pub type PointLightQuery = QueryState<(
    &'static WorldTransform,
    Option<&'static PreviousWorldTransform>,
    &'static PointLight,
)>;

impl FrameData {
    /// Queues `instance` for `slot`, in the last write when `slot` follows it.
    fn write_instance(&mut self, slot: u32, instance: InstanceData) {
        let index = self.instances.len();
        self.instances.push(instance);
        match self.instance_writes.last_mut() {
            Some(write) if write.first_slot + write.instances.len() as u32 == slot => {
                write.instances.end += 1;
            }
            _ => self.instance_writes.push(InstanceWrite {
                first_slot: slot,
                instances: index..index + 1,
            }),
        }
    }
}

/// Instances for consecutive slots from `first_slot`, one copy on the GPU.
pub struct InstanceWrite {
    pub first_slot: u32,
    /// The Instances' range in `FrameData::instances`.
    pub instances: Range<usize>,
}

/// One Primitive drawn for one Instance.
pub struct Draw {
    /// The Primitive's range in the Mesh pool's index buffer.
    pub indices: Range<u32>,
    /// The Primitive's first vertex in the Mesh pool's vertex buffer.
    pub base_vertex: i32,
    /// The Instance's slot in the Instance buffer.
    pub instance_index: u32,
}

/// Each Renderable's slot in the Instance buffer, kept until it stops being a
/// Renderable; a freed slot is taken again before the buffer grows.
#[derive(Default)]
pub struct InstanceSlots {
    by_entity: bevy_ecs::entity::EntityHashMap<u32>,
    free: Vec<u32>,
    /// One past the highest slot taken.
    end: u32,
}

impl InstanceSlots {
    /// `entity`'s slot, taking one if it has none, and whether it is new.
    fn get_or_take(&mut self, entity: Entity) -> (u32, bool) {
        if let Some(&slot) = self.by_entity.get(&entity) {
            return (slot, false);
        }
        let slot = self.free.pop().unwrap_or(self.end);
        self.end = self.end.max(slot + 1);
        self.by_entity.insert(entity, slot);
        (slot, true)
    }

    fn free(&mut self, entity: Entity) {
        if let Some(slot) = self.by_entity.remove(&entity) {
            self.free.push(slot);
        }
    }
}

/// Fills `frame` from the World's camera, lights and Renderables, seen at
/// `aspect_ratio`, Instances anchored and lights placed relative to the
/// camera (ADR 0014), reusing its lists' memory. Whatever the fixed schedule
/// moves is drawn blended between its two newest Ticks (ADR 0022). A
/// Renderable whose Mesh is not in `mesh_pool` is skipped.
///
/// Only new Renderables, blended ones and those whose World transform or
/// Material changed since the last `World::clear_trackers` write their
/// Instance, so the caller clears the trackers once per frame, after
/// preparing it.
pub fn prepare_frame(
    world: &mut World,
    point_light_query: &mut PointLightQuery,
    instance_slots: &mut InstanceSlots,
    mesh_pool: &MeshPool,
    aspect_ratio: f32,
    frame: &mut FrameData,
) {
    let camera = world.resource::<Camera>();
    let camera_position = camera.position;
    let view_proj = projection_matrix(aspect_ratio) * camera.view_matrix();
    let (anchor, offset) = anchor_and_offset(camera_position);
    frame.camera = CameraUniform {
        view_proj: view_proj.to_cols_array_2d(),
        anchor,
        offset,
        ..CameraUniform::default()
    };
    let blend_weight = world.resource::<FixedTime>().blend_weight;
    frame.light = prepare_lights(world, point_light_query, camera_position, blend_weight);

    frame.instances.clear();
    frame.instance_writes.clear();
    frame.draws.clear();
    let mut renderable_query = world.query::<(
        Entity,
        Ref<WorldTransform>,
        Option<&PreviousWorldTransform>,
        &MeshRef,
        Ref<MaterialRef>,
    )>();
    let renderables = renderable_query.query(world);
    // Slots of entities that stopped being Renderables are freed before new
    // Renderables take slots.
    for entity in world
        .removed::<WorldTransform>()
        .chain(world.removed::<MeshRef>())
        .chain(world.removed::<MaterialRef>())
    {
        if !renderables.contains(entity) {
            instance_slots.free(entity);
        }
    }
    for (entity, current, previous, mesh_ref, material_ref) in renderables.iter() {
        let (slot, is_new) = instance_slots.get_or_take(entity);
        // A blended Renderable moves every frame, between Ticks too.
        if is_new || current.is_changed() || previous.is_some() || material_ref.is_changed() {
            let transform = current.drawn(previous, blend_weight);
            let rotation_scale = transform.0.matrix3.as_mat3();
            let (anchor, offset) = anchor_and_offset(transform.translation());
            frame.write_instance(
                slot,
                InstanceData {
                    rotation_scale: padded_columns(rotation_scale),
                    normal_matrix: padded_columns(rotation_scale.inverse().transpose()),
                    anchor,
                    material_id: material_ref.0.index(),
                    offset,
                    _pad: 0,
                },
            );
        }
        let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
            tracing::warn!(
                handle = mesh_ref.0.0,
                "MeshHandle not in pool, skipping entity"
            );
            continue;
        };
        frame
            .draws
            .extend(mesh.primitives.iter().map(|primitive| Draw {
                indices: primitive.index_offset..primitive.index_offset + primitive.index_count,
                base_vertex: primitive.vertex_offset as i32,
                instance_index: slot,
            }));
    }
    frame.instance_slot_count = instance_slots.end;
}

/// The lights, point lights at their position relative to `camera_position`;
/// past the uniform's four, the rest are dropped.
fn prepare_lights(
    world: &World,
    point_light_query: &mut PointLightQuery,
    camera_position: glam::DVec3,
    blend_weight: f32,
) -> LightUniform {
    let mut light_uniform = LightUniform {
        direction: [0.5, -1.0, -0.3, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        ambient: [1.0, 1.0, 1.0, 0.1],
        point_positions: [[0.0; 4]; 4],
        point_colors: [[0.0; 4]; 4],
        num_point_lights: [0.0; 4],
    };
    let mut count = 0;
    let capacity = light_uniform.point_positions.len();
    for (current, previous, point_light) in point_light_query.iter(world).take(capacity) {
        let transform = current.drawn(previous, blend_weight);
        let position = (transform.translation() - camera_position).as_vec3();
        light_uniform.point_positions[count] = position.extend(0.0).to_array();
        light_uniform.point_colors[count] =
            point_light.color.extend(point_light.intensity).to_array();
        count += 1;
    }
    light_uniform.num_point_lights[0] = count as f32;
    light_uniform
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::assets::{ARENA_CAPACITY_BYTES, MaterialHandle, MeshHandle};
    use crate::components::Transform;
    use crate::gpu::GpuContext;
    use crate::render::LoadStaging;
    use crate::transform::propagate_world_transforms;

    /// A World of Renderables prepared frame by frame, as `State` does.
    struct Frames {
        world: World,
        propagation: Schedule,
        point_light_query: PointLightQuery,
        instance_slots: InstanceSlots,
        mesh_pool: MeshPool,
        mesh: MeshHandle,
        frame: FrameData,
    }

    impl Frames {
        /// None, after saying so, when there is no GPU for the Mesh pool.
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
            let mut mesh_pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
            let triangle = [bytemuck::Zeroable::zeroed(); 3];
            let mesh = mesh_pool
                .load(
                    &mut LoadStaging::new(context.device.clone()),
                    &[(&triangle, &[0, 1, 2])],
                )
                .expect("a triangle fits");
            let mut world = World::new();
            world.insert_resource(Camera::new());
            world.insert_resource(FixedTime::default());
            let point_light_query = world.query();
            let mut propagation = Schedule::default();
            propagation.add_systems(propagate_world_transforms);
            Some(Self {
                world,
                propagation,
                point_light_query,
                instance_slots: InstanceSlots::default(),
                mesh_pool,
                mesh,
                frame: FrameData::default(),
            })
        }

        fn spawn(&mut self, x: f64) -> Entity {
            let renderable = (
                Transform::from_translation(glam::dvec3(x, 0.0, 0.0)),
                MeshRef(self.mesh),
                MaterialRef(MaterialHandle(0)),
            );
            self.world.spawn(renderable).id()
        }

        /// Prepares the next frame and returns the slots it wrote, in order.
        fn prepare(&mut self) -> Vec<u32> {
            self.propagation.run(&mut self.world);
            prepare_frame(
                &mut self.world,
                &mut self.point_light_query,
                &mut self.instance_slots,
                &self.mesh_pool,
                1.0,
                &mut self.frame,
            );
            self.world.clear_trackers();
            self.frame
                .instance_writes
                .iter()
                .flat_map(|write| write.first_slot..write.first_slot + write.instances.len() as u32)
                .collect()
        }
    }

    #[test]
    fn renderables_keep_their_slots_and_freed_slots_are_taken_again() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let [first, second, third] = [0.0, 1.0, 2.0].map(|x| frames.spawn(x));
        assert_eq!(frames.prepare(), [0, 1, 2]);
        // Consecutive slots go to the GPU as one copy.
        assert_eq!(frames.frame.instance_writes.len(), 1);
        assert_eq!(frames.frame.instance_slot_count, 3);
        assert!(frames.prepare().is_empty());

        frames.world.despawn(second);
        assert!(frames.prepare().is_empty());

        // The freed slot is taken again; the others keep theirs.
        frames.spawn(3.0);
        assert_eq!(frames.prepare(), [1]);
        assert_eq!(frames.frame.instances[0].offset, [3.0, 0.0, 0.0]);
        assert_eq!(frames.frame.instance_slot_count, 3);
        for (entity, slot) in [(first, 0), (third, 2)] {
            frames
                .world
                .get_mut::<Transform>(entity)
                .unwrap()
                .translation
                .z = 1.0;
            assert_eq!(frames.prepare(), [slot]);
        }

        // An entity that loses its Mesh is no Renderable: it frees its slot.
        frames.world.entity_mut(first).remove::<MeshRef>();
        frames.spawn(4.0);
        assert_eq!(frames.prepare(), [0]);

        // With no slot free, a spawn grows the buffer by one.
        frames.spawn(5.0);
        assert_eq!(frames.prepare(), [3]);
        assert_eq!(frames.frame.instance_slot_count, 4);
    }

    #[test]
    fn a_changed_transform_or_material_writes_exactly_its_slot() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let [_, second, third] = [0.0, 1.0, 2.0].map(|x| frames.spawn(x));
        frames.prepare();
        let child = frames.spawn(1.0);
        frames.world.entity_mut(child).insert(ChildOf(second));
        assert_eq!(frames.prepare(), [3]);

        // Moving a parent writes its child too: the child's own Transform is
        // unchanged, but its World transform moved.
        frames
            .world
            .get_mut::<Transform>(second)
            .unwrap()
            .translation
            .y = 5.0;
        let mut written = frames.prepare();
        written.sort_unstable();
        assert_eq!(written, [1, 3]);
        let offsets: Vec<[f32; 3]> = frames
            .frame
            .instances
            .iter()
            .map(|instance| instance.offset)
            .collect();
        assert!(offsets.contains(&[1.0, 5.0, 0.0]) && offsets.contains(&[2.0, 5.0, 0.0]));

        frames.world.get_mut::<MaterialRef>(third).unwrap().0 = MaterialHandle(7);
        assert_eq!(frames.prepare(), [2]);
        assert_eq!(frames.frame.instances[0].material_id, 7);

        // A blended Renderable is written every frame: it moves between Ticks.
        let blended = frames.spawn(6.0);
        frames
            .world
            .entity_mut(blended)
            .insert(PreviousWorldTransform::default());
        assert_eq!(frames.prepare(), [4]);
        assert_eq!(frames.prepare(), [4]);
    }
}
