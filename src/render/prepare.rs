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

/// The point lights, with what blends the moving ones.
type PointLightQuery = QueryState<(
    &'static WorldTransform,
    Option<&'static PreviousWorldTransform>,
    &'static PointLight,
)>;

/// The Renderables, with what blends the moving ones and the changes that
/// write their Instance.
type RenderableQuery = QueryState<(
    Entity,
    Ref<'static, WorldTransform>,
    Option<&'static PreviousWorldTransform>,
    Ref<'static, MeshRef>,
    Option<Ref<'static, MaterialRef>>,
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
    /// The Primitive's first vertex in the Mesh pool's vertex Arena.
    pub base_vertex: i32,
    /// The Instance's slot in the Instance buffer.
    pub instance_index: u32,
}

/// Each Renderable's slot in the Instance buffer, kept until it stops being a
/// Renderable; a freed slot is taken again before the buffer grows.
#[derive(Default)]
struct InstanceSlots {
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

/// Prepares each frame's Frame data from the World, keeping across frames
/// what makes that cheap: the Renderables' Instance slots, the queries and
/// the Frame data's memory.
pub struct FramePreparer {
    point_light_query: PointLightQuery,
    renderable_query: RenderableQuery,
    instance_slots: InstanceSlots,
    frame: FrameData,
}

impl FramePreparer {
    pub fn new(world: &mut World) -> Self {
        Self {
            point_light_query: world.query(),
            renderable_query: world.query(),
            instance_slots: InstanceSlots::default(),
            frame: FrameData::default(),
        }
    }

    /// Fills the Frame data from the World's camera, lights and Renderables,
    /// seen at `aspect_ratio`, Instances anchored and lights placed relative
    /// to the camera (ADR 0014). Whatever the fixed schedule moves is drawn
    /// blended between its two newest Ticks (ADR 0022). A Renderable draws
    /// with its Material Handle, else with the Material its Mesh's Primitives
    /// name (`Mesh::material`); one whose Mesh is not in `mesh_pool`, or that
    /// has neither, is skipped.
    ///
    /// Only new Renderables, blended ones and those whose World transform,
    /// Mesh or Material changed since the last `prepare` write their
    /// Instance. It ends the World's change window (`World::clear_trackers`),
    /// so it is the frame's last World read, once per frame.
    pub fn prepare(
        &mut self,
        world: &mut World,
        mesh_pool: &MeshPool,
        aspect_ratio: f32,
    ) -> &FrameData {
        let Self {
            point_light_query,
            renderable_query,
            instance_slots,
            frame,
        } = self;
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
        let renderables = renderable_query.query(world);
        // Slots of entities that stopped being Renderables are freed before new
        // Renderables take slots.
        for entity in world
            .removed::<WorldTransform>()
            .chain(world.removed::<MeshRef>())
        {
            if !renderables.contains(entity) {
                instance_slots.free(entity);
            }
        }
        // Those that lost their Material Handle now draw with their Mesh's
        // Material.
        let lost_material_handle: bevy_ecs::entity::EntityHashSet =
            world.removed::<MaterialRef>().collect();
        for (entity, current, previous, mesh_ref, material_ref) in renderables.iter() {
            let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
                tracing::warn!(
                    handle = mesh_ref.0.0,
                    "MeshHandle not in pool, skipping entity"
                );
                continue;
            };
            let Some(material) = material_ref
                .as_ref()
                .map(|material_ref| material_ref.0)
                .or(mesh.material)
            else {
                tracing::warn!(
                    ?entity,
                    "Renderable has no Material Handle and its Mesh's Primitives name no Material, skipping it"
                );
                continue;
            };
            let (slot, is_new) = instance_slots.get_or_take(entity);
            // A blended Renderable moves every frame, between Ticks too.
            if is_new
                || current.is_changed()
                || previous.is_some()
                || (mesh_ref.is_changed() && material_ref.is_none())
                || material_ref.is_some_and(|material_ref| material_ref.is_changed())
                || lost_material_handle.contains(&entity)
            {
                let transform = current.drawn(previous, blend_weight);
                let rotation_scale = transform.0.matrix3.as_mat3();
                let (anchor, offset) = anchor_and_offset(transform.translation());
                frame.write_instance(
                    slot,
                    InstanceData {
                        rotation_scale: padded_columns(rotation_scale),
                        normal_matrix: padded_columns(rotation_scale.inverse().transpose()),
                        anchor,
                        material_id: material.index(),
                        offset,
                        _pad: 0,
                    },
                );
            }
            frame
                .draws
                .extend(mesh.primitives.iter().map(|primitive| Draw {
                    indices: primitive.index_offset..primitive.index_offset + primitive.index_count,
                    base_vertex: primitive.vertex_offset as i32,
                    instance_index: slot,
                }));
        }
        frame.instance_slot_count = instance_slots.end;
        // Ends the window in which prepare sees changes and removals, and
        // drains the removals, which otherwise pile up.
        world.clear_trackers();
        frame
    }

    /// The Frame data the last `prepare` filled.
    pub fn frame(&self) -> &FrameData {
        &self.frame
    }

    /// Makes the next `prepare` write every Renderable's Instance, for when
    /// the Renderer lost Instance writes.
    pub fn rewrite_all_instances(&mut self) {
        self.instance_slots = InstanceSlots::default();
    }
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
    use crate::assets::pools::tests::mesh;
    use crate::assets::{ARENA_CAPACITY_BYTES, MaterialHandle, MeshHandle};
    use crate::components::Transform;
    use crate::gpu::GpuContext;
    use crate::render::LoadStaging;
    use crate::transform::propagate_world_transforms;

    /// The Material `Frames::mesh` draws with.
    const MESH_MATERIAL: MaterialHandle = MaterialHandle(5);

    /// A World of Renderables prepared frame by frame, as `State` does.
    struct Frames {
        world: World,
        propagation: Schedule,
        staging: LoadStaging,
        mesh_pool: MeshPool,
        /// A triangle drawn with `MESH_MATERIAL`.
        mesh: MeshHandle,
        preparer: FramePreparer,
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
            let mut staging = LoadStaging::new(context.device.clone());
            let mut mesh_pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
            let mesh = mesh_pool
                .load(&mut staging, &mesh(3, vec![0, 1, 2]), Some(MESH_MATERIAL))
                .expect("a triangle fits");
            let mut world = World::new();
            world.insert_resource(Camera::new());
            world.insert_resource(FixedTime::default());
            let preparer = FramePreparer::new(&mut world);
            let mut propagation = Schedule::default();
            propagation.add_systems(propagate_world_transforms);
            Some(Self {
                world,
                propagation,
                staging,
                mesh_pool,
                mesh,
                preparer,
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
            self.preparer
                .prepare(&mut self.world, &self.mesh_pool, 1.0)
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
        assert_eq!(frames.preparer.frame().instance_writes.len(), 1);
        assert_eq!(frames.preparer.frame().instance_slot_count, 3);
        assert!(frames.prepare().is_empty());

        frames.world.despawn(second);
        assert!(frames.prepare().is_empty());

        // The freed slot is taken again; the others keep theirs.
        frames.spawn(3.0);
        assert_eq!(frames.prepare(), [1]);
        assert_eq!(frames.preparer.frame().instances[0].offset, [3.0, 0.0, 0.0]);
        assert_eq!(frames.preparer.frame().instance_slot_count, 3);
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
        assert_eq!(frames.preparer.frame().instance_slot_count, 4);
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
            .preparer
            .frame()
            .instances
            .iter()
            .map(|instance| instance.offset)
            .collect();
        assert!(offsets.contains(&[1.0, 5.0, 0.0]) && offsets.contains(&[2.0, 5.0, 0.0]));

        frames.world.get_mut::<MaterialRef>(third).unwrap().0 = MaterialHandle(7);
        assert_eq!(frames.prepare(), [2]);
        assert_eq!(frames.preparer.frame().instances[0].material_id, 7);

        // A blended Renderable is written every frame: it moves between Ticks.
        let blended = frames.spawn(6.0);
        frames
            .world
            .entity_mut(blended)
            .insert(PreviousWorldTransform::default());
        assert_eq!(frames.prepare(), [4]);
        assert_eq!(frames.prepare(), [4]);
    }

    #[test]
    fn a_renderable_draws_with_its_material_handle_else_with_its_meshs_material() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let [other_mesh, bare_mesh] = [Some(MaterialHandle(6)), None].map(|material| {
            let mesh =
                frames
                    .mesh_pool
                    .load(&mut frames.staging, &mesh(3, vec![0, 1, 2]), material);
            mesh.expect("a triangle fits")
        });
        let entity = frames
            .world
            .spawn((Transform::IDENTITY, MeshRef(frames.mesh)))
            .id();
        // The Material IDs of the Instances the next frame writes.
        let written_material_ids = |frames: &mut Frames| -> Vec<u32> {
            frames.prepare();
            let instances = &frames.preparer.frame().instances;
            instances
                .iter()
                .map(|instance| instance.material_id)
                .collect()
        };
        assert_eq!(written_material_ids(&mut frames), [MESH_MATERIAL.index()]);

        frames
            .world
            .entity_mut(entity)
            .insert(MaterialRef(MaterialHandle(7)));
        assert_eq!(written_material_ids(&mut frames), [7]);
        // Under its Handle, another Mesh leaves its Instance as it was.
        frames.world.entity_mut(entity).insert(MeshRef(other_mesh));
        assert!(written_material_ids(&mut frames).is_empty());
        // Without it, it draws with its Mesh's Material, and so after a
        // change of Mesh.
        frames.world.entity_mut(entity).remove::<MaterialRef>();
        assert_eq!(written_material_ids(&mut frames), [6]);
        frames.world.entity_mut(entity).insert(MeshRef(frames.mesh));
        assert_eq!(written_material_ids(&mut frames), [MESH_MATERIAL.index()]);
        // With neither Material, it is not drawn.
        frames.world.entity_mut(entity).insert(MeshRef(bare_mesh));
        assert!(written_material_ids(&mut frames).is_empty());
        assert!(frames.preparer.frame().draws.is_empty());
    }

    #[test]
    fn rewriting_all_instances_writes_every_slot_once() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        for x in [0.0, 1.0, 2.0] {
            frames.spawn(x);
        }
        frames.prepare();
        assert!(frames.prepare().is_empty());

        frames.preparer.rewrite_all_instances();
        assert_eq!(frames.prepare(), [0, 1, 2]);
        assert_eq!(frames.preparer.frame().instance_slot_count, 3);
        assert!(frames.prepare().is_empty());
    }
}
