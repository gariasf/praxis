use std::ops::Range;

use bevy_ecs::prelude::*;

use crate::assets::MeshPool;
use crate::camera::{Camera, projection_matrix};
use crate::components::{DirectionalLight, MaterialRef, MeshRef, PointLight};
use crate::render::instance::{anchor_and_offset, padded_columns};
use crate::render::{
    CameraUniform, INSTANCE_CAPACITY, InstanceData, LightUniform, NO_MATERIAL, NO_MESH,
};
use crate::time::FixedTime;
use crate::transform::{PreviousWorldTransform, WorldTransform};

/// Everything the renderer draws in one frame, owned, so it holds no borrow
/// into the ECS World and can cross to another thread (ADRs 0004, 0016). It
/// lists no draws: the GPU builds the frame's draw from the Instances.
#[derive(Default)]
pub struct FrameData {
    pub camera: CameraUniform,
    pub light: LightUniform,
    /// Slots the Instance buffer holds: one past the highest any Renderable
    /// has taken. The GPU draws each, but a freed one.
    pub instance_slot_count: u32,
    /// The Meshlets of the Instances in those slots, and their triangles'
    /// indices: the most Meshlet records and packed indices the frame's draw
    /// holds, where culling leaves every Instance in.
    pub meshlet_count: u64,
    pub index_count: u64,
    /// The Instances this frame writes, in `instance_writes` order; every
    /// other slot keeps what an earlier frame wrote (ADR 0003).
    pub instances: Vec<InstanceData>,
    pub instance_writes: Vec<InstanceWrite>,
}

const _: () = {
    const fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<FrameData>()
};

/// The lights of one kind, with what blends the moving ones.
type LightQuery<Light> = QueryState<(
    &'static WorldTransform,
    Option<&'static PreviousWorldTransform>,
    &'static Light,
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

/// Each Renderable's slot in the Instance buffer, kept until it stops being a
/// Renderable or is not drawn; a slot freed in an earlier frame is taken
/// again before a fresh one, and once all `INSTANCE_CAPACITY` are taken a
/// Renderable is refused one.
#[derive(Default)]
struct InstanceSlots {
    by_entity: bevy_ecs::entity::EntityHashMap<u32>,
    free: Vec<u32>,
    /// Slots freed this frame, which take an Instance with no Mesh before
    /// they join `free`.
    freed: Vec<u32>,
    /// One past the highest slot taken.
    end: u32,
    /// Renderables refused a slot and not yet given one, each warned about
    /// once.
    refused: bevy_ecs::entity::EntityHashSet,
}

impl InstanceSlots {
    /// `entity`'s slot, taking one if it has none, and whether it is new; None
    /// when none is left, warned about the first time `entity` is refused.
    fn get_or_take(&mut self, entity: Entity) -> Option<(u32, bool)> {
        if let Some(&slot) = self.by_entity.get(&entity) {
            return Some((slot, false));
        }
        let fresh = (self.end < INSTANCE_CAPACITY).then_some(self.end);
        let Some(slot) = self.free.pop().or(fresh) else {
            if self.refused.insert(entity) {
                tracing::warn!(
                    ?entity,
                    capacity = INSTANCE_CAPACITY,
                    "Renderable refused: the Instance buffer is full"
                );
            }
            return None;
        };
        self.end = self.end.max(slot + 1);
        self.by_entity.insert(entity, slot);
        self.refused.remove(&entity);
        Some((slot, true))
    }

    fn free(&mut self, entity: Entity) {
        self.refused.remove(&entity);
        if let Some(slot) = self.by_entity.remove(&entity) {
            self.freed.push(slot);
        }
    }
}

/// Prepares each frame's Frame data from the World, keeping across frames
/// what makes that cheap: the Renderables' Instance slots, the queries and
/// the Frame data's memory.
pub struct FramePreparer {
    directional_light_query: LightQuery<DirectionalLight>,
    point_light_query: LightQuery<PointLight>,
    /// Set while the ECS World holds directional lights past the first,
    /// which were warned about when it was set.
    warned_of_undrawn_directional_lights: bool,
    renderable_query: RenderableQuery,
    instance_slots: InstanceSlots,
    /// Set by `rewrite_all_instances` until the next `prepare`.
    rewrite_every_instance: bool,
    renderables_refused: u32,
    frame: FrameData,
}

impl FramePreparer {
    pub fn new(world: &mut World) -> Self {
        Self {
            directional_light_query: world.query(),
            point_light_query: world.query(),
            warned_of_undrawn_directional_lights: false,
            renderable_query: world.query(),
            instance_slots: InstanceSlots::default(),
            rewrite_every_instance: false,
            renderables_refused: 0,
            frame: FrameData::default(),
        }
    }

    /// Fills the Frame data from the World's camera, lights and Renderables,
    /// seen at `aspect_ratio`, Instances anchored and lights placed relative
    /// to the camera (ADR 0014). Whatever the fixed schedule moves is drawn
    /// blended between its two newest Ticks (ADR 0022). A Renderable draws
    /// each Primitive of its Mesh with its Material Handle, else with the
    /// Material the Primitive names; one whose Mesh is not in `mesh_pool`, or
    /// that has no Material Handle where a Primitive names no Material, is
    /// skipped and frees its slot. One that finds the
    /// Instance buffer full is refused, warned about once and counted every
    /// frame, and neither written nor drawn until a slot frees.
    ///
    /// Only new Renderables, blended ones and those whose World transform,
    /// Mesh or Material changed since the last `prepare` write their
    /// Instance, and a slot freed since writes an Instance with no Mesh. It
    /// ends the World's change window (`World::clear_trackers`), so it is the
    /// frame's last World read, once per frame.
    pub fn prepare(
        &mut self,
        world: &mut World,
        mesh_pool: &MeshPool,
        aspect_ratio: f32,
    ) -> &FrameData {
        let Self {
            directional_light_query,
            point_light_query,
            warned_of_undrawn_directional_lights,
            renderable_query,
            instance_slots,
            rewrite_every_instance,
            renderables_refused,
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
        frame.light = prepare_lights(
            world,
            directional_light_query,
            point_light_query,
            warned_of_undrawn_directional_lights,
            camera_position,
            blend_weight,
        );

        frame.instances.clear();
        frame.instance_writes.clear();
        frame.meshlet_count = 0;
        frame.index_count = 0;
        *renderables_refused = 0;
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
        // Materials.
        let lost_material_handle: bevy_ecs::entity::EntityHashSet =
            world.removed::<MaterialRef>().collect();
        for (entity, current, previous, mesh_ref, material_ref) in renderables.iter() {
            let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
                tracing::warn!(
                    handle = mesh_ref.0.0,
                    "MeshHandle not in pool, skipping entity"
                );
                instance_slots.free(entity);
                continue;
            };
            if material_ref.is_none() && mesh.needs_material_handle {
                tracing::warn!(
                    ?entity,
                    "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it"
                );
                instance_slots.free(entity);
                continue;
            }
            let Some((slot, is_new)) = instance_slots.get_or_take(entity) else {
                *renderables_refused += 1;
                continue;
            };
            // A blended Renderable moves every frame, between Ticks too.
            if *rewrite_every_instance
                || is_new
                || current.is_changed()
                || previous.is_some()
                || mesh_ref.is_changed()
                || material_ref
                    .as_ref()
                    .is_some_and(|material_ref| material_ref.is_changed())
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
                        material_id: material_ref
                            .map_or(NO_MATERIAL, |material_ref| material_ref.0.index()),
                        offset,
                        mesh: mesh_ref.0.0,
                    },
                );
            }
            frame.meshlet_count += u64::from(mesh.meshlet_count);
            frame.index_count += u64::from(mesh.index_count);
        }
        // A slot freed this frame, or every free slot when every Instance is
        // written again, takes an Instance with no Mesh, so the GPU no longer
        // draws it; Renderables may take it from the next frame.
        let rewritten_free_slots = if *rewrite_every_instance {
            instance_slots.free.as_slice()
        } else {
            &[]
        };
        for &slot in instance_slots.freed.iter().chain(rewritten_free_slots) {
            let freed = InstanceData {
                mesh: NO_MESH,
                ..bytemuck::Zeroable::zeroed()
            };
            frame.write_instance(slot, freed);
        }
        instance_slots.free.append(&mut instance_slots.freed);
        frame.instance_slot_count = instance_slots.end;
        *rewrite_every_instance = false;
        // Ends the window in which prepare sees changes and removals, and
        // drains the removals, which otherwise pile up.
        world.clear_trackers();
        frame
    }

    /// The Frame data the last `prepare` filled.
    pub fn frame(&self) -> &FrameData {
        &self.frame
    }

    /// Renderables the last `prepare` refused a slot because the Instance
    /// buffer is full, so neither written nor drawn; each is retried every
    /// frame.
    pub fn renderables_refused(&self) -> u32 {
        self.renderables_refused
    }

    /// Makes the next `prepare` write every Renderable's Instance into the
    /// slot it keeps, for when the Renderer lost Instance writes.
    pub fn rewrite_all_instances(&mut self) {
        self.rewrite_every_instance = true;
    }
}

/// The lights: the first directional light the query yields, shining along
/// its entity's forward axis, the others not drawn and warned about when they
/// appear; and point lights at their position relative to `camera_position`,
/// past the uniform's four dropped.
fn prepare_lights(
    world: &World,
    directional_light_query: &mut LightQuery<DirectionalLight>,
    point_light_query: &mut LightQuery<PointLight>,
    warned_of_undrawn_directional_lights: &mut bool,
    camera_position: glam::DVec3,
    blend_weight: f32,
) -> LightUniform {
    let mut light_uniform = LightUniform {
        ambient: [1.0, 1.0, 1.0, 0.1],
        ..LightUniform::default()
    };
    let mut directional_lights = directional_light_query.iter(world);
    if let Some((current, previous, directional_light)) = directional_lights.next() {
        let transform = current.drawn(previous, blend_weight);
        // A zero scale, or a blend half way through a half turn, leaves no
        // direction to shade with, so the light is not drawn.
        if let Some(direction) = (transform.0.matrix3 * glam::DVec3::NEG_Z).try_normalize() {
            light_uniform.direction = direction.as_vec3().extend(0.0).to_array();
            light_uniform.color = directional_light
                .color
                .extend(directional_light.intensity)
                .to_array();
            light_uniform.directional_light_count = 1;
        }
    }
    let undrawn = directional_lights.count();
    if undrawn > 0 && !*warned_of_undrawn_directional_lights {
        tracing::warn!(
            undrawn,
            "Directional lights past the first are not drawn: the light uniform holds one"
        );
    }
    *warned_of_undrawn_directional_lights = undrawn > 0;
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
    light_uniform.point_light_count = count as u32;
    light_uniform
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::assets::pools::tests::{logged, mesh};
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
                .load(
                    &mut staging,
                    &mesh(3, vec![0, 1, 2]),
                    &[Some(MESH_MATERIAL)],
                )
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

        // A freed slot is written an Instance with no Mesh, which draws
        // nothing.
        frames.world.despawn(second);
        assert_eq!(frames.prepare(), [1]);
        assert_eq!(frames.preparer.frame().instances[0].mesh, NO_MESH);
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

        // An entity that loses its Mesh is no Renderable: it frees its slot,
        // which no Renderable takes before the next frame, so a spawn in the
        // same frame takes a fresh one.
        frames.world.entity_mut(first).remove::<MeshRef>();
        frames.spawn(4.0);
        assert_eq!(frames.prepare(), [3, 0]);
        frames.spawn(5.0);
        assert_eq!(frames.prepare(), [0]);
    }

    #[test]
    fn a_renderable_past_capacity_is_refused_and_warned_about_once_until_a_slot_frees() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let filled: Vec<Entity> = (0..INSTANCE_CAPACITY).map(|_| frames.spawn(0.0)).collect();
        let slots: Vec<u32> = (0..INSTANCE_CAPACITY).collect();
        assert_eq!(frames.prepare(), slots);

        // Refused frame after frame: warned about once, counted every frame,
        // and given no slot, so neither written nor drawn.
        let past_capacity = frames.spawn(3.0);
        let ((), log) = logged(|| {
            for _ in 0..3 {
                let written = frames.prepare();
                assert!(written.is_empty(), "past capacity, wrote slots {written:?}");
                assert_eq!(frames.preparer.renderables_refused(), 1);
                assert_eq!(
                    frames.preparer.frame().meshlet_count,
                    u64::from(INSTANCE_CAPACITY)
                );
            }
            // Rewriting every Instance keeps every slot where it was.
            frames.preparer.rewrite_all_instances();
            assert_eq!(frames.prepare(), slots);
            assert_eq!(frames.preparer.renderables_refused(), 1);
        });
        assert_eq!(log.matches("Renderable refused").count(), 1, "{log:?}");
        assert!(log.contains(&format!("{past_capacity:?}")), "{log:?}");

        // It takes the slot another Renderable leaves, from the frame after.
        frames.world.despawn(filled[7]);
        assert_eq!(frames.prepare(), [7]);
        assert_eq!(frames.preparer.frame().instances[0].mesh, NO_MESH);
        assert_eq!(frames.preparer.renderables_refused(), 1);
        assert_eq!(frames.prepare(), [7]);
        assert_eq!(frames.preparer.frame().instances[0].offset, [3.0, 0.0, 0.0]);
        assert_eq!(frames.preparer.renderables_refused(), 0);
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
    fn an_instance_names_its_mesh_and_draws_with_its_material_handle_else_its_meshs() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let [other_mesh, bare_mesh] = [Some(MaterialHandle(6)), None].map(|material| {
            let mesh =
                frames
                    .mesh_pool
                    .load(&mut frames.staging, &mesh(3, vec![0, 1, 2]), &[material]);
            mesh.expect("a triangle fits")
        });
        let first_mesh = frames.mesh;
        let entity = frames
            .world
            .spawn((Transform::IDENTITY, MeshRef(first_mesh)))
            .id();
        // The Mesh and the Material ID of each Instance the next frame writes.
        let written = |frames: &mut Frames| -> Vec<(u32, u32)> {
            frames.prepare();
            let instances = &frames.preparer.frame().instances;
            instances
                .iter()
                .map(|instance| (instance.mesh, instance.material_id))
                .collect()
        };
        assert_eq!(written(&mut frames), [(first_mesh.0, NO_MATERIAL)]);

        frames
            .world
            .entity_mut(entity)
            .insert(MaterialRef(MaterialHandle(7)));
        assert_eq!(written(&mut frames), [(first_mesh.0, 7)]);
        frames.world.entity_mut(entity).insert(MeshRef(other_mesh));
        assert_eq!(written(&mut frames), [(other_mesh.0, 7)]);
        // Without its Handle, it draws with its Mesh's Materials, and so
        // after a change of Mesh.
        frames.world.entity_mut(entity).remove::<MaterialRef>();
        assert_eq!(written(&mut frames), [(other_mesh.0, NO_MATERIAL)]);
        frames.world.entity_mut(entity).insert(MeshRef(first_mesh));
        assert_eq!(written(&mut frames), [(first_mesh.0, NO_MATERIAL)]);
        // With a Primitive that names no Material, it is not drawn, and its
        // slot draws nothing.
        frames.world.entity_mut(entity).insert(MeshRef(bare_mesh));
        assert_eq!(written(&mut frames), [(NO_MESH, 0)]);
        assert_eq!(frames.preparer.frame().meshlet_count, 0);
    }

    #[test]
    fn a_scene_without_a_directional_light_puts_none_in_frame_data() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        frames.prepare();
        assert_eq!(frames.preparer.frame().light.directional_light_count, 0);
    }

    #[test]
    fn a_directional_light_scaled_to_nothing_is_not_drawn() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        let flattened = Transform {
            scale: glam::Vec3::ZERO,
            ..Transform::IDENTITY
        };
        let light = DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 1.0,
        };
        frames.world.spawn((flattened, light));
        frames.prepare();
        assert_eq!(frames.preparer.frame().light.directional_light_count, 0);
    }

    #[test]
    fn a_second_directional_light_is_warned_about_once_and_not_drawn() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        for color in [glam::Vec3::X, glam::Vec3::Y] {
            let light = DirectionalLight {
                color,
                intensity: 2.0,
            };
            frames.world.spawn((Transform::IDENTITY, light));
        }
        let ((), log) = logged(|| {
            for _ in 0..3 {
                frames.prepare();
                let light = &frames.preparer.frame().light;
                assert_eq!(light.directional_light_count, 1);
                assert_eq!(light.color, [1.0, 0.0, 0.0, 2.0]);
            }
        });
        assert_eq!(log.matches("not drawn").count(), 1, "{log:?}");
    }

    #[test]
    fn a_directional_light_shines_along_its_forward_axis_blended_between_ticks() {
        let Some(mut frames) = Frames::new() else {
            return;
        };
        // A quarter turn about +X takes the forward axis (-Z) straight down;
        // half a Tick on from an unturned Tick, it is drawn half way there.
        frames.world.resource_mut::<FixedTime>().blend_weight = 0.5;
        let turned = Transform {
            rotation: glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ..Transform::IDENTITY
        };
        let light = DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 1.0,
        };
        let previous = PreviousWorldTransform(Some(glam::DAffine3::IDENTITY));
        frames.world.spawn((turned, light, previous));
        frames.prepare();
        let direction = frames.preparer.frame().light.direction;
        let half_way = -std::f32::consts::FRAC_1_SQRT_2;
        let expected = [0.0, half_way, half_way, 0.0];
        assert!(
            direction
                .iter()
                .zip(expected)
                .all(|(component, expected)| (component - expected).abs() <= 1e-6),
            "{direction:?}, not {expected:?}"
        );
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
        assert!(frames.prepare().is_empty());
    }
}
