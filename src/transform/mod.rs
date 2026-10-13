use bevy_ecs::prelude::*;

use crate::components::Transform;

/// Where an entity sits in the Game world: its `Transform` composed with
/// every ancestor's, in f64 (ADR 0014). Derived by
/// `propagate_world_transforms`, never authored.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct WorldTransform(pub glam::DAffine3);

impl WorldTransform {
    pub fn translation(&self) -> glam::DVec3 {
        self.0.translation
    }

    /// The model matrix relative to `origin`: the translation difference is
    /// taken in f64, then cast, so the matrix holds no World position in f32.
    pub fn matrix_relative_to(&self, origin: glam::DVec3) -> glam::Mat4 {
        glam::Mat4::from_mat3_translation(
            self.0.matrix3.as_mat3(),
            (self.0.translation - origin).as_vec3(),
        )
    }
}

impl Transform {
    fn to_affine(self) -> glam::DAffine3 {
        glam::DAffine3::from_scale_rotation_translation(
            self.scale.as_dvec3(),
            self.rotation.as_dquat(),
            self.translation,
        )
    }
}

/// An entity whose Transform or parent changed.
type Moved = Or<(Changed<Transform>, Changed<ChildOf>)>;

/// Recomputes the World transform of every entity whose own or an
/// ancestor's Transform or parent changed since this system last ran, and
/// of no other entity.
#[allow(clippy::too_many_arguments)]
pub fn propagate_world_transforms(
    moved: Query<Entity, Moved>,
    mut orphaned: RemovedComponents<ChildOf>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
    children: Query<&Children>,
    mut world_transforms: Query<&mut WorldTransform>,
    mut dirty: Local<Vec<Entity>>,
    mut stack: Local<Vec<(Entity, glam::DAffine3)>>,
) {
    dirty.clear();
    dirty.extend(moved.iter());
    dirty.extend(orphaned.read());
    dirty.sort_unstable();
    dirty.dedup();

    for &entity in dirty.iter() {
        // A dirty ancestor recomputes this entity's whole subtree.
        if parents
            .iter_ancestors(entity)
            .any(|ancestor| dirty.binary_search(&ancestor).is_ok())
        {
            continue;
        }
        let parent_world = parents
            .get(entity)
            .ok()
            .and_then(|child_of| world_transforms.get(child_of.parent()).ok())
            .map_or(glam::DAffine3::IDENTITY, |parent| parent.0);
        stack.push((entity, parent_world));
        while let Some((entity, parent_world)) = stack.pop() {
            // A despawned entity, or one without a Transform, ends its branch.
            let (Ok(transform), Ok(mut world_transform)) =
                (transforms.get(entity), world_transforms.get_mut(entity))
            else {
                continue;
            };
            let world = parent_world * transform.to_affine();
            world_transform.0 = world;
            if let Ok(entity_children) = children.get(entity) {
                stack.extend(entity_children.iter().map(|child| (child, world)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many World transforms the last propagation wrote.
    #[derive(Resource, Default)]
    struct Propagated(usize);

    fn count_propagated(
        written: Query<(), Changed<WorldTransform>>,
        mut propagated: ResMut<Propagated>,
    ) {
        propagated.0 = written.iter().count();
    }

    fn propagation() -> (World, Schedule) {
        let mut world = World::new();
        world.init_resource::<Propagated>();
        let mut schedule = Schedule::default();
        schedule.add_systems((propagate_world_transforms, count_propagated).chain());
        (world, schedule)
    }

    /// 120° about (1, 1, 1), which takes x to y, y to z and z to x, and is
    /// exact in f32, so the closed form below is exact too.
    const CYCLE: glam::Quat = glam::Quat::from_xyzw(0.5, 0.5, 0.5, 0.5);

    /// 100 km out: an f32 composition misses the oracle there by millimetres.
    const ROOT: glam::DVec3 = glam::dvec3(100_000.1, 0.25, -3.3);

    /// A root at `ROOT` and four descendants, each rotated by `CYCLE`, scaled
    /// by 2 and, below the root, 1 m along its parent's x.
    fn spawn_chain(world: &mut World) -> [Entity; 5] {
        let link = |translation| Transform {
            translation,
            rotation: CYCLE,
            scale: glam::Vec3::splat(2.0),
        };
        let mut chain = [world.spawn(link(ROOT)).id(); 5];
        for depth in 1..5 {
            chain[depth] = world
                .spawn((link(glam::DVec3::X), ChildOf(chain[depth - 1])))
                .id();
        }
        chain
    }

    fn world_position(world: &World, entity: Entity) -> glam::DVec3 {
        world.get::<WorldTransform>(entity).unwrap().translation()
    }

    fn assert_near(actual: glam::DVec3, expected: glam::DVec3) {
        assert!(
            (actual - expected).abs().max_element() <= 1e-9,
            "{actual} is not within 1e-9 m of {expected}"
        );
    }

    #[test]
    fn a_five_deep_chain_matches_its_closed_form() {
        let (mut world, mut schedule) = propagation();
        let chain = spawn_chain(&mut world);
        schedule.run(&mut world);

        // Depth d adds (2 CYCLE)^d x: 2y, 4z, 8x, then 16y.
        assert_near(
            world_position(&world, chain[4]),
            ROOT + glam::dvec3(8.0, 18.0, 4.0),
        );
        // Below depth 4, x has turned five times, to z, and grown by 2^5.
        let point = world
            .get::<WorldTransform>(chain[4])
            .unwrap()
            .0
            .transform_point3(glam::DVec3::X);
        assert_near(point, ROOT + glam::dvec3(8.0, 18.0, 36.0));
    }

    #[test]
    fn a_frame_without_changes_propagates_nothing() {
        let (mut world, mut schedule) = propagation();
        let chain = spawn_chain(&mut world);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 5);

        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 0);

        // A change reaches its entity and descendants only.
        world.get_mut::<Transform>(chain[2]).unwrap().translation = glam::DVec3::Y;
        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 3);
        assert_near(
            world_position(&world, chain[2]),
            ROOT + glam::dvec3(4.0, 2.0, 0.0),
        );
    }

    #[test]
    fn a_reparented_entity_follows_its_new_parent() {
        let (mut world, mut schedule) = propagation();
        let chain = spawn_chain(&mut world);
        let other = world
            .spawn(Transform::from_translation(glam::dvec3(-5.0, 0.0, 0.0)))
            .id();
        schedule.run(&mut world);

        world.entity_mut(chain[3]).insert(ChildOf(other));
        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 2);
        assert_near(
            world_position(&world, chain[3]),
            glam::dvec3(-4.0, 0.0, 0.0),
        );
        // chain[4] is 2 CYCLE x below chain[3].
        assert_near(
            world_position(&world, chain[4]),
            glam::dvec3(-4.0, 2.0, 0.0),
        );

        // Detached, it is a root: its Transform is its World transform.
        world.entity_mut(chain[3]).remove::<ChildOf>();
        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 2);
        assert_near(world_position(&world, chain[3]), glam::DVec3::X);
    }

    #[test]
    fn despawning_a_parent_despawns_its_subtree() {
        let (mut world, mut schedule) = propagation();
        let chain = spawn_chain(&mut world);
        schedule.run(&mut world);

        world.despawn(chain[2]);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Propagated>().0, 0);
        for entity in &chain[2..] {
            assert!(world.get_entity(*entity).is_err());
        }
        assert_near(
            world_position(&world, chain[1]),
            ROOT + glam::dvec3(0.0, 2.0, 0.0),
        );
    }
}
