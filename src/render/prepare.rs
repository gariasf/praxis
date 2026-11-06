use std::ops::Range;

use bevy_ecs::prelude::*;

use crate::assets::LoadedMeshes;
use crate::assets::pools::Generations;
use crate::camera::{Camera, projection_matrix};
use crate::components::{Bend, DirectionalLight, MaterialRef, MeshRef, PointLight};
use crate::environment::Environment;
use crate::render::bend_pass::{BEND_CAPACITY, BendData};
use crate::render::instance::{anchor_and_offset, padded_columns};
use crate::render::{
    CameraUniform, DYNAMIC_TLAS_CAPACITY, INSTANCE_CAPACITY, InstanceData, LIGHT_CAPACITY,
    LightData, LightUniform, NO_MATERIAL, NO_MESH,
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
    /// The light buffer's lights, directional lights first, at most
    /// `LIGHT_CAPACITY`.
    pub lights: Vec<LightData>,
    /// The colour the lit pass clears to: the scene's background luminance,
    /// pre-exposed as the lit pass exposes what it draws.
    pub background: wgpu::Color,
    /// Slots the Instance buffer holds: one past the highest any Renderable
    /// has taken. The GPU draws each, but a freed one.
    pub instance_slot_count: u32,
    /// The Instances this frame writes, in `instance_writes` order; every
    /// other slot keeps what an earlier frame wrote (ADR 0003).
    pub instances: Vec<InstanceData>,
    pub instance_writes: Vec<InstanceWrite>,
    /// The Moving Instances the dynamic TLAS holds, at most
    /// `DYNAMIC_TLAS_CAPACITY`, those in the lowest slots, in no order. Each
    /// is written every frame, so each is among `instances`.
    pub moving_instances: Vec<MovingInstance>,
    /// The Meshes the GPU bends this frame (`Bend`), at most
    /// `BEND_CAPACITY`, in the order of their ranges of the vertex Arena.
    pub bends: Vec<BendData>,
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

/// The Renderables whose Mesh the GPU bends.
type BendQuery = QueryState<(&'static MeshRef, &'static Bend)>;

/// What a Renderable's Instance holds, and what blends a moving one.
type RenderableData = (
    Entity,
    &'static WorldTransform,
    Option<&'static PreviousWorldTransform>,
    &'static MeshRef,
    Option<&'static MaterialRef>,
);

/// One Renderable a `RenderableQuery` found.
type RenderableItem<'world> = bevy_ecs::query::QueryItem<'world, 'world, RenderableData>;

/// The Renderables.
type RenderableQuery<Filter = ()> = QueryState<RenderableData, Filter>;

/// The Renderables whose World transform, Mesh or Material changed since the
/// last `prepare`, new ones included, and the blended ones, which move every
/// frame. Of those prepare has written, only these and those that lost their
/// Material Handle or previous World transform are written again.
type ChangedRenderableQuery = RenderableQuery<
    Or<(
        Changed<WorldTransform>,
        Changed<MeshRef>,
        Changed<MaterialRef>,
        With<PreviousWorldTransform>,
    )>,
>;

impl FrameData {
    /// Queues `instance` for `slot`, a Moving Instance or not, in the last
    /// write when `slot` follows it and it is of the same kind, and lists it
    /// among the Moving Instances if it is one.
    fn write_instance(&mut self, slot: u32, instance: InstanceData, moving: bool) {
        let index = self.instances.len();
        self.instances.push(instance);
        if moving {
            self.moving_instances.push(MovingInstance {
                slot,
                instance: index,
            });
        }
        match self.instance_writes.last_mut() {
            Some(write)
                if write.moving == moving
                    && write.first_slot + write.instances.len() as u32 == slot =>
            {
                write.instances.end += 1;
            }
            _ => self.instance_writes.push(InstanceWrite {
                first_slot: slot,
                instances: index..index + 1,
                moving,
            }),
        }
    }
}

/// Instances for consecutive slots from `first_slot`, one copy on the GPU.
pub struct InstanceWrite {
    pub first_slot: u32,
    /// The Instances' range in `FrameData::instances`.
    pub instances: Range<usize>,
    /// Whether they are Moving Instances, which the static TLAS leaves out.
    pub moving: bool,
}

/// A Moving Instance the dynamic TLAS holds: its slot, and its Instance's
/// index in `FrameData::instances`.
pub struct MovingInstance {
    pub slot: u32,
    pub instance: usize,
}

/// The Instance of a slot that draws nothing: a freed slot's, or a skipped
/// Renderable's.
fn no_mesh_instance() -> InstanceData {
    InstanceData {
        mesh: NO_MESH,
        ..bytemuck::Zeroable::zeroed()
    }
}

/// Each Renderable's slot in the Instance buffer, kept until it stops being a
/// Renderable (ADR 0003); a slot freed in an earlier frame is taken
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
    /// `entity`'s slot, taking one if it has none; None when none is left,
    /// warned about the first time `entity` is refused.
    fn get_or_take(&mut self, entity: Entity) -> Option<u32> {
        if let Some(&slot) = self.by_entity.get(&entity) {
            return Some(slot);
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
        Some(slot)
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
    /// Each point light's squared distance from the camera and its index in
    /// query order, kept for its memory.
    point_light_distances: Vec<(f64, usize)>,
    lights_overflowed: u32,
    /// Set while lights overflow the light buffer, which was warned about
    /// when it was set.
    warned_of_overflowed_lights: bool,
    renderable_query: RenderableQuery,
    changed_renderable_query: ChangedRenderableQuery,
    instance_slots: InstanceSlots,
    /// Renderables the last `prepare` skipped or refused a slot, so wrote no
    /// Instance with a Mesh for: visited again, changed or not.
    unwritten: bevy_ecs::entity::EntityHashSet,
    /// Renderables the last `prepare` skipped, whose slots hold an Instance
    /// with no Mesh.
    skipped: bevy_ecs::entity::EntityHashSet,
    /// Set by `rewrite_all_instances` until the next `prepare`.
    rewrite_every_instance: bool,
    /// The Mesh and Material pools' frees the last `prepare` saw.
    pool_frees: u64,
    renderables_refused: u32,
    moving_instances_left_out: u32,
    /// Set while Moving Instances are left out of the dynamic TLAS, which
    /// was warned about when it was set.
    warned_of_moving_instances_left_out: bool,
    bend_query: BendQuery,
    bends_left_out: u32,
    /// Set while bends are left out of the bend pass, which was warned about
    /// when it was set.
    warned_of_bends_left_out: bool,
    frame: FrameData,
}

impl FramePreparer {
    pub fn new(world: &mut World) -> Self {
        Self {
            directional_light_query: world.query(),
            point_light_query: world.query(),
            point_light_distances: Vec::new(),
            lights_overflowed: 0,
            warned_of_overflowed_lights: false,
            renderable_query: world.query(),
            changed_renderable_query: world.query_filtered(),
            instance_slots: InstanceSlots::default(),
            unwritten: bevy_ecs::entity::EntityHashSet::default(),
            skipped: bevy_ecs::entity::EntityHashSet::default(),
            rewrite_every_instance: false,
            pool_frees: 0,
            renderables_refused: 0,
            moving_instances_left_out: 0,
            warned_of_moving_instances_left_out: false,
            bend_query: world.query(),
            bends_left_out: 0,
            warned_of_bends_left_out: false,
            frame: FrameData::default(),
        }
    }

    /// Fills the Frame data from the World's camera, lights, Environment and
    /// Renderables, seen at `aspect_ratio`, Instances and point lights
    /// anchored (ADR 0014); lights past the light buffer's capacity are
    /// counted, warned about when they appear, and not drawn. Whatever the
    /// fixed schedule moves is drawn blended between its two newest Ticks
    /// (ADR 0022). A Renderable draws each Primitive of its Mesh with its
    /// Material Handle, else with the Material the Primitive names; one whose
    /// Mesh is not in `meshes`, whose Material Handle `material_generations`
    /// finds no Material for, or that has no Material Handle where a
    /// Primitive names no Material, is skipped with a warning: it keeps its
    /// slot, which holds an Instance with no Mesh until the Renderable is
    /// drawn. One that finds the Instance buffer full is refused, warned
    /// about once and counted every frame, and neither written nor drawn
    /// until a slot frees.
    /// An Instance is a Moving Instance while its entity carries its previous
    /// World transform; past the dynamic TLAS's capacity, those in the
    /// highest slots are left out of it, counted, and warned about when they
    /// appear. A Renderable with a `Bend` has its Mesh bent from the rest
    /// Mesh when both are in `meshes` with as many vertices, and is
    /// warned about otherwise, as is a bend that would race with another;
    /// past the bend pass's capacity, the bends of the Meshes highest in the
    /// vertex Arena are left out, counted, and warned about when they
    /// appear.
    ///
    /// Only new Renderables, blended ones and those whose World transform,
    /// Mesh or Material changed since the last `prepare` write their
    /// Instance, every one when either pool freed an entry since, and a slot
    /// freed since writes an Instance with no Mesh. It
    /// ends the World's change window (`World::clear_trackers`), so it is the
    /// frame's last World read, once per frame.
    pub fn prepare(
        &mut self,
        world: &mut World,
        meshes: &LoadedMeshes,
        material_generations: &Generations,
        aspect_ratio: f32,
    ) -> &FrameData {
        let Self {
            directional_light_query,
            point_light_query,
            point_light_distances,
            lights_overflowed,
            warned_of_overflowed_lights,
            renderable_query,
            changed_renderable_query,
            instance_slots,
            unwritten,
            skipped,
            rewrite_every_instance,
            pool_frees,
            renderables_refused,
            moving_instances_left_out,
            warned_of_moving_instances_left_out,
            bend_query,
            bends_left_out,
            warned_of_bends_left_out,
            frame,
        } = self;
        let camera = world.resource::<Camera>();
        let camera_position = camera.position;
        let view_proj = projection_matrix(aspect_ratio) * camera.view_matrix();
        let (anchor, offset) = anchor_and_offset(camera_position);
        let exposure = camera.exposure();
        frame.camera = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            anchor,
            exposure,
            offset,
            ..CameraUniform::default()
        };
        let environment = *world.resource::<Environment>();
        let background = environment.background_luminance * exposure;
        frame.background = wgpu::Color {
            r: background.x.into(),
            g: background.y.into(),
            b: background.z.into(),
            a: 1.0,
        };
        let blend_weight = world.resource::<FixedTime>().blend_weight;
        let directional_light_count = prepare_lights(
            world,
            directional_light_query,
            point_light_query,
            camera_position,
            blend_weight,
            &mut frame.lights,
            point_light_distances,
            lights_overflowed,
        );
        frame.light = LightUniform {
            ambient_illuminance: environment.ambient_illuminance.extend(0.0).to_array(),
            directional_light_count,
            point_light_count: frame.lights.len() as u32 - directional_light_count,
            ..LightUniform::default()
        };
        if *lights_overflowed > 0 && !*warned_of_overflowed_lights {
            tracing::warn!(
                lights_overflowed = *lights_overflowed,
                capacity = LIGHT_CAPACITY,
                "Lights past the light buffer's capacity are not drawn"
            );
        }
        *warned_of_overflowed_lights = *lights_overflowed > 0;

        let frees = meshes.frees() + material_generations.frees();
        if std::mem::replace(pool_frees, frees) != frees {
            *rewrite_every_instance = true;
        }
        frame.instances.clear();
        frame.instance_writes.clear();
        frame.moving_instances.clear();
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
        // Materials, and those that lost their previous World transform are
        // static, so the static TLAS takes them.
        let lost_material_handle_or_previous_transform: bevy_ecs::entity::EntityHashSet = world
            .removed::<MaterialRef>()
            .chain(world.removed::<PreviousWorldTransform>())
            .collect();
        let retried = std::mem::take(unwritten);
        let skipped_before = std::mem::take(skipped);
        let mut visit = |(entity, current, previous, mesh_ref, material_ref): RenderableItem<
            '_,
        >| {
            let drawn = match meshes.get(mesh_ref.0) {
                None => {
                    tracing::warn!(
                        handle = ?mesh_ref.0,
                        "MeshHandle not in pool, skipping entity"
                    );
                    false
                }
                Some(_)
                    if material_ref.is_some_and(|material_ref| {
                        !material_generations.live(material_ref.0.index, material_ref.0.generation)
                    }) =>
                {
                    tracing::warn!(
                        ?entity,
                        "Renderable's Material Handle is not in the pool, skipping it"
                    );
                    false
                }
                Some(mesh) if material_ref.is_none() && mesh.needs_material_handle => {
                    tracing::warn!(
                        ?entity,
                        "Renderable has no Material Handle and a Primitive of its Mesh names no Material, skipping it"
                    );
                    false
                }
                Some(_) => true,
            };
            let Some(slot) = instance_slots.get_or_take(entity) else {
                *renderables_refused += 1;
                unwritten.insert(entity);
                return;
            };
            if !drawn {
                // Its slot draws nothing until its Mesh does; an Instance with
                // no Mesh written there before stays.
                if *rewrite_every_instance || !skipped_before.contains(&entity) {
                    frame.write_instance(slot, no_mesh_instance(), false);
                }
                skipped.insert(entity);
                unwritten.insert(entity);
                return;
            }
            // Each Renderable visited is new, changed, blended, retried or lost
            // a component, or every Instance is written again.
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
                    mesh: mesh_ref.0.index,
                },
                previous.is_some(),
            );
        };
        if *rewrite_every_instance {
            renderables.iter().for_each(&mut visit);
        } else {
            // An unchanged Renderable keeps its Instance, unless the last
            // prepare skipped or refused it, or it lost its Material Handle or
            // previous World transform.
            let changed_renderables = changed_renderable_query.query(world);
            changed_renderables.iter().for_each(&mut visit);
            let lost = lost_material_handle_or_previous_transform
                .iter()
                .filter(|entity| !retried.contains(*entity));
            for &entity in retried.iter().chain(lost) {
                if !changed_renderables.contains(entity)
                    && let Ok(renderable) = renderables.get(entity)
                {
                    visit(renderable);
                }
            }
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
            frame.write_instance(slot, no_mesh_instance(), false);
        }
        instance_slots.free.append(&mut instance_slots.freed);
        frame.instance_slot_count = instance_slots.end;
        // Past the dynamic TLAS's capacity, the lowest slots keep their place.
        let capacity = DYNAMIC_TLAS_CAPACITY as usize;
        *moving_instances_left_out = frame.moving_instances.len().saturating_sub(capacity) as u32;
        if *moving_instances_left_out > 0 {
            frame
                .moving_instances
                .select_nth_unstable_by_key(capacity, |moving| moving.slot);
            frame.moving_instances.truncate(capacity);
            if !*warned_of_moving_instances_left_out {
                tracing::warn!(
                    left_out = *moving_instances_left_out,
                    capacity,
                    "Moving Instances left out of the dynamic TLAS, past its capacity: those in the highest slots cast no shadow"
                );
            }
        }
        *warned_of_moving_instances_left_out = *moving_instances_left_out > 0;
        prepare_bends(
            world,
            bend_query,
            meshes,
            blend_weight,
            &mut frame.bends,
            bends_left_out,
        );
        if *bends_left_out > 0 && !*warned_of_bends_left_out {
            tracing::warn!(
                left_out = *bends_left_out,
                capacity = BEND_CAPACITY,
                "Bends left out of the bend pass, past its capacity: their Meshes keep the shape they last took"
            );
        }
        *warned_of_bends_left_out = *bends_left_out > 0;
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

    /// Lights the last `prepare` left out of the light buffer, which had no
    /// room for them, so not drawn.
    pub fn lights_overflowed(&self) -> u32 {
        self.lights_overflowed
    }

    /// Moving Instances the last `prepare` left out of the dynamic TLAS,
    /// which had no room for them, so they cast no shadow.
    pub fn moving_instances_left_out(&self) -> u32 {
        self.moving_instances_left_out
    }

    /// Bends the last `prepare` left out of the bend pass, which had no room
    /// for them, so their Meshes keep the shape they last took.
    pub fn bends_left_out(&self) -> u32 {
        self.bends_left_out
    }

    /// Makes the next `prepare` write every Renderable's Instance into the
    /// slot it keeps, for when the Renderer lost Instance writes.
    pub fn rewrite_all_instances(&mut self) {
        self.rewrite_every_instance = true;
    }
}

/// Fills `lights` with every directional light, shining along its entity's
/// forward axis, then every point light, at its World position as an Anchor
/// plus an offset (ADR 0014), each blended between its two newest Ticks, up
/// to `LIGHT_CAPACITY`. Point lights past it are cut farthest from
/// `camera_position` first, the rest keeping their query order. Returns how
/// many are directional, and sets `overflowed` to how many the light buffer
/// had no room for.
#[expect(clippy::too_many_arguments)]
fn prepare_lights(
    world: &World,
    directional_light_query: &mut LightQuery<DirectionalLight>,
    point_light_query: &mut LightQuery<PointLight>,
    camera_position: glam::DVec3,
    blend_weight: f32,
    lights: &mut Vec<LightData>,
    point_light_distances: &mut Vec<(f64, usize)>,
    overflowed: &mut u32,
) -> u32 {
    let mut directional_lights =
        directional_light_query
            .iter(world)
            .filter_map(|(current, previous, light)| {
                let transform = current.drawn(previous, blend_weight);
                // A zero scale leaves no direction to shade with, so the
                // light is not drawn.
                let direction = (transform.0.matrix3 * glam::DVec3::NEG_Z).try_normalize()?;
                Some(LightData {
                    offset: direction.as_vec3().to_array(),
                    intensity: (light.color * light.intensity).to_array(),
                    ..LightData::default()
                })
            });
    let point_lights = point_light_query
        .iter(world)
        .map(|(current, previous, light)| {
            let transform = current.drawn(previous, blend_weight);
            let position = transform.translation();
            let (anchor, offset) = anchor_and_offset(position);
            let light = LightData {
                anchor,
                range: light.range,
                offset,
                intensity: (light.color * light.intensity).to_array(),
                ..LightData::default()
            };
            (position.distance_squared(camera_position), light)
        });
    let capacity = LIGHT_CAPACITY as usize;
    lights.clear();
    lights.extend(directional_lights.by_ref().take(capacity));
    let directional_light_count = lights.len();
    let directional_lights_overflowed = directional_lights.count();
    point_light_distances.clear();
    for (index, (distance_squared, light)) in point_lights.enumerate() {
        point_light_distances.push((distance_squared, index));
        lights.push(light);
    }
    let point_light_count = point_light_distances.len();
    let room = capacity - directional_light_count;
    if point_light_count > room {
        point_light_distances.select_nth_unstable_by(room, |left, right| {
            left.0.total_cmp(&right.0).then(left.1.cmp(&right.1))
        });
        point_light_distances.truncate(room);
        point_light_distances.sort_unstable_by_key(|&(_, index)| index);
        for (kept, &(_, index)) in point_light_distances.iter().enumerate() {
            lights[directional_light_count + kept] = lights[directional_light_count + index];
        }
        lights.truncate(capacity);
    }
    *overflowed = (directional_lights_overflowed + point_light_count.saturating_sub(room)) as u32;
    directional_light_count as u32
}

/// Fills `bends` with a bend per Renderable with a `Bend` whose Mesh and
/// rest Mesh `meshes` holds with as many vertices, its curvature blended
/// `blend_weight` of the way from the previous Tick's, in the order of the
/// bent Meshes' ranges of the vertex Arena, up to `BEND_CAPACITY`. The pass
/// writes each bent Mesh while it reads each rest Mesh, so of two bends of
/// one Mesh it keeps one, and it drops a bend from a Mesh that a bend
/// writes, its own included, warning about both. Sets `left_out` to how
/// many it had no room for.
fn prepare_bends(
    world: &World,
    bend_query: &mut BendQuery,
    meshes: &LoadedMeshes,
    blend_weight: f32,
    bends: &mut Vec<BendData>,
    left_out: &mut u32,
) {
    bends.clear();
    for (mesh_ref, bend) in bend_query.iter(world) {
        let (Some(bent), Some(rest)) = (meshes.get(mesh_ref.0), meshes.get(bend.rest)) else {
            tracing::warn!(
                mesh = ?mesh_ref.0,
                rest = ?bend.rest,
                "A bent Mesh or its rest Mesh is not in the pool, so it is not bent"
            );
            continue;
        };
        if bent.vertices.len() != rest.vertices.len() {
            tracing::warn!(
                mesh = ?mesh_ref.0,
                rest = ?bend.rest,
                "A bent Mesh and its rest Mesh hold different vertex counts, so it is not bent"
            );
            continue;
        }
        let curvature = bend.drawn_curvature(blend_weight);
        bends.push(BendData {
            rest_first_vertex: rest.vertices.start,
            first_vertex: bent.vertices.start,
            vertex_count: bent.vertices.len() as u32,
            azimuth: bend.azimuth,
            curvature,
        });
    }
    bends.sort_unstable_by_key(|bend| bend.first_vertex);
    let listed = bends.len();
    bends.dedup_by_key(|bend| bend.first_vertex);
    let mut index = 0;
    while index < bends.len() {
        let rest = bends[index].rest_first_vertex;
        if bends
            .binary_search_by_key(&rest, |bend| bend.first_vertex)
            .is_ok()
        {
            bends.remove(index);
        } else {
            index += 1;
        }
    }
    if bends.len() < listed {
        tracing::warn!(
            dropped = listed - bends.len(),
            "Bends that would race are dropped: a Mesh two Renderables bend is bent once, and one bent from a Mesh a bend writes is not bent"
        );
    }
    let capacity = BEND_CAPACITY as usize;
    *left_out = bends.len().saturating_sub(capacity) as u32;
    bends.truncate(capacity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::pools::tests::logged;
    use crate::assets::{MaterialHandle, Mesh, MeshHandle};
    use crate::components::Transform;
    use crate::render::DYNAMIC_TLAS_CAPACITY;
    use crate::transform::propagate_world_transforms;

    /// A World of Renderables prepared frame by frame, as `State` does.
    struct Frames {
        world: World,
        propagation: Schedule,
        meshes: LoadedMeshes,
        /// Materials 0 to 7.
        material_generations: Generations,
        /// Vertices of the Meshes loaded, each Mesh's after the last's.
        vertices_loaded: u32,
        /// A Mesh of three vertices whose Primitives each name a Material.
        mesh: MeshHandle,
        preparer: FramePreparer,
    }

    impl Frames {
        fn new() -> Self {
            let mut material_generations = Generations::default();
            for _ in 0..8 {
                material_generations.take();
            }
            let mut meshes = LoadedMeshes::default();
            let mesh = meshes.insert(Mesh {
                needs_material_handle: false,
                vertices: 0..3,
                bounds: [[0.0; 3]; 2],
                texel_densities: Vec::new(),
            });
            let mut world = World::new();
            world.insert_resource(Camera::new());
            world.insert_resource(FixedTime::default());
            world.insert_resource(Environment::default());
            let preparer = FramePreparer::new(&mut world);
            let mut propagation = Schedule::default();
            propagation.add_systems(propagate_world_transforms);
            Self {
                world,
                propagation,
                meshes,
                material_generations,
                vertices_loaded: 3,
                mesh,
                preparer,
            }
        }

        fn spawn(&mut self, x: f64) -> Entity {
            let renderable = (
                Transform::from_translation(glam::dvec3(x, 0.0, 0.0)),
                MeshRef(self.mesh),
                MaterialRef(MaterialHandle {
                    index: 0,
                    generation: 0,
                }),
            );
            self.world.spawn(renderable).id()
        }

        /// Prepares the next frame and returns the slots it wrote, in order.
        fn prepare(&mut self) -> Vec<u32> {
            self.propagation.run(&mut self.world);
            self.preparer
                .prepare(
                    &mut self.world,
                    &self.meshes,
                    &self.material_generations,
                    1.0,
                )
                .instance_writes
                .iter()
                .flat_map(|write| write.first_slot..write.first_slot + write.instances.len() as u32)
                .collect()
        }
    }

    #[test]
    fn renderables_keep_their_slots_and_freed_slots_are_taken_again() {
        let mut frames = Frames::new();
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
        let mut frames = Frames::new();
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
        let mut frames = Frames::new();
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

        frames.world.get_mut::<MaterialRef>(third).unwrap().0 = MaterialHandle {
            index: 7,
            generation: 0,
        };
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
        // Its write is flagged a Moving Instance's, apart from a static
        // Instance's in the next slot.
        frames.spawn(7.0);
        frames.prepare();
        let mut writes: Vec<(u32, usize, bool)> = frames
            .preparer
            .frame()
            .instance_writes
            .iter()
            .map(|write| (write.first_slot, write.instances.len(), write.moving))
            .collect();
        writes.sort_unstable();
        assert_eq!(writes, [(4, 1, true), (5, 1, false)]);
    }

    /// The slots of the Moving Instances the last frame listed for the
    /// dynamic TLAS, in order.
    fn listed(frames: &Frames) -> Vec<u32> {
        let mut slots: Vec<u32> = frames
            .preparer
            .frame()
            .moving_instances
            .iter()
            .map(|moving| moving.slot)
            .collect();
        slots.sort_unstable();
        slots
    }

    #[test]
    fn an_instance_is_moving_while_its_entity_carries_its_previous_world_transform() {
        let mut frames = Frames::new();
        let [still, moving] = [0.0, 1.0].map(|x| frames.spawn(x));
        frames
            .world
            .entity_mut(moving)
            .insert(PreviousWorldTransform::default());
        frames.prepare();
        // Listed with its own Instance, and written every frame.
        let frame = frames.preparer.frame();
        let [entry] = &frame.moving_instances[..] else {
            panic!(
                "{} Moving Instances listed, not one",
                frame.moving_instances.len()
            );
        };
        assert_eq!(frame.instances[entry.instance].offset, [1.0, 0.0, 0.0]);
        let (moving_slot, still_slot) = (entry.slot, 1 - entry.slot);
        assert_eq!(frames.prepare(), [moving_slot]);
        assert_eq!(listed(&frames), [moving_slot]);

        // Losing the component makes it static in that frame: written once
        // more, unflagged, so the static TLAS takes it, and no longer listed.
        frames
            .world
            .entity_mut(moving)
            .remove::<PreviousWorldTransform>();
        assert_eq!(frames.prepare(), [moving_slot]);
        assert!(!frames.preparer.frame().instance_writes[0].moving);
        assert!(listed(&frames).is_empty());
        assert!(frames.prepare().is_empty());

        frames
            .world
            .entity_mut(still)
            .insert(PreviousWorldTransform::default());
        assert_eq!(frames.prepare(), [still_slot]);
        assert_eq!(listed(&frames), [still_slot]);
    }

    /// One Moving Instance more than the dynamic TLAS holds: the lowest slots
    /// keep their place in it, so the one in the highest slot is left out,
    /// counted every frame and warned about once, whatever order the ECS World
    /// holds them in.
    #[test]
    fn the_moving_instance_in_the_highest_slot_past_the_dynamic_tlas_capacity_is_left_out() {
        let mut frames = Frames::new();
        let spawn_moving = |frames: &mut Frames| {
            let entity = frames.spawn(0.0);
            let mut entity = frames.world.entity_mut(entity);
            entity.insert(PreviousWorldTransform::default()).id()
        };
        let moving: Vec<Entity> = (0..=DYNAMIC_TLAS_CAPACITY)
            .map(|_| spawn_moving(&mut frames))
            .collect();
        let lowest: Vec<u32> = (0..DYNAMIC_TLAS_CAPACITY).collect();
        let ((), log) = logged(|| {
            for _ in 0..3 {
                frames.prepare();
                assert_eq!(listed(&frames), lowest);
                assert_eq!(frames.preparer.moving_instances_left_out(), 1);
            }
        });
        let warnings = log.matches("left out of the dynamic TLAS").count();
        assert_eq!(warnings, 1, "{log:?}");

        // Slot 3 frees, so all fit. A new Moving Instance takes it again: the
        // ECS World now holds it last, yet the highest slot is the one left
        // out.
        frames.world.despawn(moving[3]);
        frames.prepare();
        assert_eq!(frames.preparer.moving_instances_left_out(), 0);
        assert_eq!(listed(&frames).len(), DYNAMIC_TLAS_CAPACITY as usize);
        spawn_moving(&mut frames);
        frames.prepare();
        assert_eq!(listed(&frames), lowest);
        assert_eq!(frames.preparer.moving_instances_left_out(), 1);
    }

    #[test]
    fn an_instance_names_its_mesh_and_draws_with_its_material_handle_else_its_meshs() {
        let mut frames = Frames::new();
        let other_mesh = load(&mut frames, 3);
        let bare_mesh = frames.meshes.insert(Mesh {
            needs_material_handle: true,
            vertices: 0..0,
            bounds: [[0.0; 3]; 2],
            texel_densities: Vec::new(),
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
        assert_eq!(written(&mut frames), [(first_mesh.index, NO_MATERIAL)]);

        frames
            .world
            .entity_mut(entity)
            .insert(MaterialRef(MaterialHandle {
                index: 7,
                generation: 0,
            }));
        assert_eq!(written(&mut frames), [(first_mesh.index, 7)]);
        frames.world.entity_mut(entity).insert(MeshRef(other_mesh));
        assert_eq!(written(&mut frames), [(other_mesh.index, 7)]);
        // Without its Handle, it draws with its Mesh's Materials, and so
        // after a change of Mesh.
        frames.world.entity_mut(entity).remove::<MaterialRef>();
        assert_eq!(written(&mut frames), [(other_mesh.index, NO_MATERIAL)]);
        frames.world.entity_mut(entity).insert(MeshRef(first_mesh));
        assert_eq!(written(&mut frames), [(first_mesh.index, NO_MATERIAL)]);
        // With a Primitive that names no Material, it is not drawn, and its
        // slot draws nothing.
        frames.world.entity_mut(entity).insert(MeshRef(bare_mesh));
        assert_eq!(written(&mut frames), [(NO_MESH, 0)]);
    }

    #[test]
    fn a_renderable_whose_mesh_is_not_in_the_pool_keeps_its_slot_and_draws_there_once_it_loads() {
        let mut frames = Frames::new();
        // The slots the next frame writes, in slot order, and whether each
        // Instance names a Mesh.
        let written = |frames: &mut Frames| -> Vec<(u32, bool)> {
            let slots = frames.prepare();
            let instances = &frames.preparer.frame().instances;
            let mut slots_and_meshes: Vec<(u32, bool)> = slots
                .into_iter()
                .zip(instances.iter().map(|instance| instance.mesh != NO_MESH))
                .collect();
            slots_and_meshes.sort_unstable();
            slots_and_meshes
        };
        frames.spawn(0.0);
        let switched = frames.spawn(1.0);
        assert_eq!(written(&mut frames), [(0, true), (1, true)]);

        // The Handle the next load returns.
        let unloaded = MeshHandle {
            index: frames.mesh.index + 1,
            generation: 0,
        };
        // A Renderable whose Mesh is not in the pool keeps its slot, which is
        // written once an Instance with no Mesh; so is a new one's slot.
        frames.world.entity_mut(switched).insert(MeshRef(unloaded));
        assert_eq!(written(&mut frames), [(1, false)]);
        frames.world.spawn((Transform::IDENTITY, MeshRef(unloaded)));
        assert_eq!(written(&mut frames), [(2, false)]);
        for _ in 0..2 {
            assert!(written(&mut frames).is_empty());
        }
        // Neither slot is free for another Renderable to take.
        frames.spawn(3.0);
        assert_eq!(written(&mut frames), [(3, true)]);
        assert_eq!(frames.preparer.frame().instance_slot_count, 4);
        // Every Instance written again writes theirs again with no Mesh.
        frames.preparer.rewrite_all_instances();
        assert_eq!(
            written(&mut frames),
            [(0, true), (1, false), (2, false), (3, true)]
        );

        // Unchanged since, both draw in their slots as soon as their Mesh is
        // in the pool.
        let loaded = load(&mut frames, 3);
        assert!(loaded == unloaded, "the pool loaded a different handle");
        assert_eq!(written(&mut frames), [(1, true), (2, true)]);
        let instances = &frames.preparer.frame().instances;
        assert!(
            instances
                .iter()
                .all(|instance| instance.mesh == unloaded.index)
        );
        assert!(written(&mut frames).is_empty());
        assert_eq!(frames.preparer.frame().instance_slot_count, 4);
    }

    #[test]
    fn a_renderable_whose_handle_was_freed_is_skipped_with_a_warning_and_keeps_its_slot() {
        let mut frames = Frames::new();
        // The slots the next frame writes, in slot order, whether each
        // Instance names a Mesh, and what the frame logged.
        let written = |frames: &mut Frames| -> (Vec<(u32, bool)>, String) {
            let (slots, log) = logged(|| frames.prepare());
            let instances = &frames.preparer.frame().instances;
            let mut slots_and_meshes: Vec<(u32, bool)> = slots
                .into_iter()
                .zip(instances.iter().map(|instance| instance.mesh != NO_MESH))
                .collect();
            slots_and_meshes.sort_unstable();
            (slots_and_meshes, log)
        };
        let freed_mesh = frames.mesh;
        frames.spawn(0.0);
        let other_mesh = load(&mut frames, 3);
        let other_material = MaterialHandle {
            index: 1,
            generation: 0,
        };
        frames.world.spawn((
            Transform::IDENTITY,
            MeshRef(other_mesh),
            MaterialRef(other_material),
        ));
        assert_eq!(written(&mut frames).0, [(0, true), (1, true)]);
        assert!(written(&mut frames).0.is_empty());

        // Unchanged since, each Renderable is visited again once a pool
        // frees an entry, and the one whose Mesh was freed is skipped, though
        // another Mesh took its index.
        assert!(frames.meshes.remove(freed_mesh));
        let replacement = load(&mut frames, 3);
        assert_eq!(replacement.index, freed_mesh.index);
        let (slots, log) = written(&mut frames);
        assert_eq!(slots, [(0, false), (1, true)]);
        assert!(
            log.contains("MeshHandle not in pool"),
            "not logged: {log:?}"
        );
        assert!(written(&mut frames).0.is_empty());

        assert!(
            frames
                .material_generations
                .free(other_material.index, other_material.generation)
        );
        let (slots, log) = written(&mut frames);
        assert_eq!(slots, [(0, false), (1, false)]);
        assert!(
            log.contains("Material Handle is not in the pool"),
            "not logged: {log:?}"
        );
        // Both keep their slots.
        frames
            .world
            .spawn((Transform::IDENTITY, MeshRef(replacement)));
        assert_eq!(written(&mut frames).0, [(2, true)]);
        assert_eq!(frames.preparer.frame().instance_slot_count, 3);
    }

    /// Loads into `frames` a Mesh of `vertex_count` vertices, after the last
    /// Mesh's in the vertex Arena, whose Primitives each name a Material.
    fn load(frames: &mut Frames, vertex_count: u32) -> MeshHandle {
        let first_vertex = frames.vertices_loaded;
        frames.vertices_loaded += vertex_count;
        frames.meshes.insert(Mesh {
            needs_material_handle: false,
            vertices: first_vertex..frames.vertices_loaded,
            bounds: [[0.0; 3]; 2],
            texel_densities: Vec::new(),
        })
    }

    /// A bend from `rest` towards azimuth 0.5, its curvature 0.2 after the
    /// previous Tick and `curvature` after the newest.
    fn bend(rest: MeshHandle, curvature: f32) -> Bend {
        Bend {
            rest,
            azimuth: 0.5,
            curvature,
            previous_curvature: 0.2,
        }
    }

    /// Frame data lists each bend in the order of its bent Mesh's range of
    /// the vertex Arena, whatever order the ECS World holds them in, from its
    /// rest Mesh's range, its curvature blended between the two newest Ticks.
    /// A bend whose Mesh is not in the pool, or whose rest Mesh holds other
    /// vertices, is warned about and left unbent; so are bends that would
    /// race: a second bend of one Mesh, and bends from a Mesh a bend writes,
    /// a Mesh's own included.
    #[test]
    fn each_bend_reaches_frame_data_from_its_rest_mesh_blended_between_ticks() {
        let mut frames = Frames::new();
        let rest = load(&mut frames, 3);
        let [first, second, own, third] = [(); 4].map(|()| load(&mut frames, 3));
        let longer = load(&mut frames, 4);
        frames.world.resource_mut::<FixedTime>().blend_weight = 0.25;
        for (mesh, bend) in [
            (second, bend(rest, 1.0)),
            (first, bend(rest, 0.6)),
            (first, bend(rest, 0.6)),
            (longer, bend(rest, 0.4)),
            (
                MeshHandle {
                    index: 99,
                    generation: 0,
                },
                bend(rest, 0.4),
            ),
            (own, bend(own, 0.4)),
            (third, bend(second, 0.4)),
        ] {
            frames
                .world
                .spawn((Transform::IDENTITY, MeshRef(mesh), bend));
        }
        let ((), log) = logged(|| {
            frames.prepare();
        });

        let start = |mesh| frames.meshes.get(mesh).expect("loaded").vertices.start;
        let listed: Vec<(u32, u32, u32, f32)> = frames
            .preparer
            .frame()
            .bends
            .iter()
            .map(|bend| {
                let (rest, bent, count) =
                    (bend.rest_first_vertex, bend.first_vertex, bend.vertex_count);
                assert_eq!(bend.azimuth, 0.5);
                (rest, bent, count, bend.curvature)
            })
            .collect();
        // A quarter of the way from 0.2 to 0.6, and to 1.
        let expected = [(first, 0.3), (second, 0.4)];
        assert_eq!(listed.len(), expected.len(), "{listed:?}");
        for ((rest_start, bent_start, count, curvature), (mesh, blended)) in
            listed.into_iter().zip(expected)
        {
            assert_eq!(
                (rest_start, bent_start, count),
                (start(rest), start(mesh), 3)
            );
            assert!(
                (curvature - blended).abs() <= 1e-6,
                "{curvature}, not {blended}"
            );
        }
        assert_eq!(log.matches("so it is not bent").count(), 2, "{log}");
        assert_eq!(log.matches("dropped=3").count(), 1, "{log}");
        assert_eq!(frames.preparer.bends_left_out(), 0);
    }

    /// One bend more than the bend pass holds: the bend of the Mesh highest
    /// in the vertex Arena is left out, counted every frame and warned about
    /// once.
    #[test]
    fn the_bend_of_the_highest_mesh_past_the_bend_capacity_is_left_out() {
        let mut frames = Frames::new();
        let rest = load(&mut frames, 3);
        let mut firsts: Vec<u32> = (0..=BEND_CAPACITY)
            .map(|_| {
                let mesh = load(&mut frames, 3);
                frames
                    .world
                    .spawn((Transform::IDENTITY, MeshRef(mesh), bend(rest, 1.0)));
                frames.meshes.get(mesh).expect("loaded").vertices.start
            })
            .collect();
        firsts.sort_unstable();
        firsts.pop();
        let ((), log) = logged(|| {
            for _ in 0..3 {
                frames.prepare();
                let listed: Vec<u32> = frames
                    .preparer
                    .frame()
                    .bends
                    .iter()
                    .map(|bend| bend.first_vertex)
                    .collect();
                assert_eq!(listed, firsts);
                assert_eq!(frames.preparer.bends_left_out(), 1);
            }
        });
        let warnings = log.matches("left out of the bend pass").count();
        assert_eq!(warnings, 1, "{log:?}");
    }

    #[test]
    fn a_scene_without_a_directional_light_puts_none_in_frame_data() {
        let mut frames = Frames::new();
        frames.prepare();
        assert_eq!(frames.preparer.frame().light.directional_light_count, 0);
        assert!(frames.preparer.frame().lights.is_empty());
    }

    #[test]
    fn a_directional_light_scaled_to_nothing_is_not_drawn() {
        let mut frames = Frames::new();
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

    /// Two directional lights and a point light 100 km out: both directional
    /// lights are drawn, first, each its colour times its lux, then the
    /// point light as its Anchor and offset, its colour times its candela.
    #[test]
    fn every_light_is_drawn_directional_lights_first_point_lights_anchored() {
        let mut frames = Frames::new();
        let point_light = PointLight {
            color: glam::vec3(1.0, 0.5, 0.25),
            intensity: 4.0,
            range: 3.0,
        };
        let position = glam::dvec3(100_000.25, 2.0, -64.5);
        frames
            .world
            .spawn((Transform::from_translation(position), point_light));
        for color in [glam::Vec3::X, glam::Vec3::Y] {
            let light = DirectionalLight {
                color,
                intensity: 2.0,
            };
            frames.world.spawn((Transform::IDENTITY, light));
        }
        frames.prepare();
        let frame = frames.preparer.frame();
        let counts = (
            frame.light.directional_light_count,
            frame.light.point_light_count,
        );
        assert_eq!(counts, (2, 1));
        let intensities: Vec<[f32; 3]> = frame.lights.iter().map(|light| light.intensity).collect();
        assert_eq!(
            intensities,
            [[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [4.0, 2.0, 1.0]]
        );
        let point = frame.lights[2];
        assert_eq!(
            (point.anchor, point.offset, point.range),
            ([1562, 0, -2], [32.25, 2.0, 63.5], 3.0)
        );
    }

    /// One directional light and one point light more than the light buffer
    /// holds: the directional light and all but two point lights are drawn,
    /// and the two are counted every frame and warned about once, until
    /// they fit again.
    #[test]
    fn lights_past_the_light_buffers_capacity_are_counted_and_warned_about_once() {
        let mut frames = Frames::new();
        let sun = DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 1.0,
        };
        frames.world.spawn((Transform::IDENTITY, sun));
        let point_light = PointLight {
            color: glam::Vec3::ONE,
            intensity: 1.0,
            range: 1.0,
        };
        let point_lights: Vec<Entity> = (0..LIGHT_CAPACITY + 1)
            .map(|index| {
                let position = glam::dvec3(f64::from(index), 0.0, 0.0);
                let light = (Transform::from_translation(position), point_light);
                frames.world.spawn(light).id()
            })
            .collect();
        let ((), log) = logged(|| {
            for _ in 0..3 {
                frames.prepare();
                let light = frames.preparer.frame().light;
                let counts = (light.directional_light_count, light.point_light_count);
                assert_eq!(counts, (1, LIGHT_CAPACITY - 1));
                assert_eq!(frames.preparer.lights_overflowed(), 2);
            }
        });
        assert_eq!(log.matches("light buffer's capacity").count(), 1, "{log:?}");

        for &entity in &point_lights[..2] {
            frames.world.despawn(entity);
        }
        frames.prepare();
        assert_eq!(
            frames.preparer.frame().lights.len(),
            LIGHT_CAPACITY as usize
        );
        assert_eq!(frames.preparer.lights_overflowed(), 0);
    }

    /// Three point lights more than the light buffer holds, spawned
    /// farthest first, each with its distance from the camera as its
    /// intensity: the three farthest are cut, and the rest keep the World's
    /// query order.
    #[test]
    fn past_the_light_buffers_capacity_the_farthest_point_lights_are_cut() {
        let mut frames = Frames::new();
        let camera_position = glam::dvec3(1.0e6, 0.0, 0.0);
        frames.world.resource_mut::<Camera>().position = camera_position;
        for distance in (1..=LIGHT_CAPACITY + 3).rev() {
            let position = camera_position + glam::dvec3(0.0, f64::from(distance), 0.0);
            let light = PointLight {
                color: glam::Vec3::ONE,
                intensity: distance as f32,
                range: 1.0,
            };
            frames
                .world
                .spawn((Transform::from_translation(position), light));
        }

        frames.prepare();
        let lights = &frames.preparer.frame().lights;
        let distances: Vec<f32> = lights.iter().map(|light| light.intensity[0]).collect();
        let nearest_in_query_order: Vec<f32> = frames
            .world
            .query::<&PointLight>()
            .iter(&frames.world)
            .map(|light| light.intensity)
            .filter(|&distance| distance <= LIGHT_CAPACITY as f32)
            .collect();
        assert_eq!(distances, nearest_in_query_order);
        assert_eq!(frames.preparer.lights_overflowed(), 3);
    }

    #[test]
    fn a_directional_light_shines_along_its_forward_axis_blended_between_ticks() {
        let mut frames = Frames::new();
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
        let direction = frames.preparer.frame().lights[0].offset;
        let half_way = -std::f32::consts::FRAC_1_SQRT_2;
        let expected = [0.0, half_way, half_way];
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
        let mut frames = Frames::new();
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
