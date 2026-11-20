//! Mip priority (ADR 0017): the finest mip each streamed Texture needs, the
//! finest any loaded Chunk's Placement sampling it needs from its distance
//! and its Primitive's UV texel density against the 1440p pixel footprint,
//! and the mips to request from the IO pool for it, finest-needed first,
//! from where the camera will be once a read has had time to land.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use glam::DVec3;

use crate::assets::material::EvictionRank;
use crate::assets::{LoadedMeshes, MaterialPool};
use crate::camera::FIELD_OF_VIEW;
use crate::components::Transform;
use crate::cooked::{AssetId, CHUNK_EDGE_METRES};
use crate::streaming::{LEAST_LEAD_FRAMES, LoadedChunk};

/// The view mip priority judges against: 1440 pixels tall at 16:9.
const PRIORITY_VIEW_HEIGHT_PIXELS: f64 = 1440.0;
const PRIORITY_VIEW_ASPECT_RATIO: f64 = 16.0 / 9.0;

/// Metres per metre of distance that the priority view's finest pixel
/// spans: a pixel at its corners, where a pixel's angle is narrowest, by
/// the square of the cosine of its angle off the view axis.
fn finest_pixel_metres_per_metre() -> f64 {
    let half_height = (f64::from(FIELD_OF_VIEW) / 2.0).tan();
    let corner_squared = half_height.powi(2) * (1.0 + PRIORITY_VIEW_ASPECT_RATIO.powi(2));
    2.0 * half_height / PRIORITY_VIEW_HEIGHT_PIXELS / (1.0 + corner_squared)
}

/// The finest mip a surface sampling a Texture at `texels_per_metre` of its
/// mip 0 needs, seen from `distance_metres` away: the mip whose texels are
/// no larger than the finest pixel's footprint there, face on. Trilinear
/// filtering samples no finer, nor does anisotropic filtering whose ratio
/// of anisotropy is not rounded up, as its level of detail then follows the
/// footprint's minor axis: Vulkan's `log2(ρmax / min(ρmax / ρmin,
/// maxAniso))` [ref:vulkan-anisotropy-lod].
fn finest_mip_needed(texels_per_metre: f64, distance_metres: f64) -> u32 {
    let texels_per_pixel = texels_per_metre * distance_metres * finest_pixel_metres_per_metre();
    texels_per_pixel.max(1.0).log2().floor() as u32
}

/// Frames before the camera, closing on a surface at its last frame's
/// rate, would reach it that its mips are requested: the Chunk streamer's
/// least lead.
const LEAD_FRAMES: f64 = LEAST_LEAD_FRAMES as f64;

/// How a loaded Chunk or Far proxy is seen this frame, for its mips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sight {
    /// Drawn: a mip it needs that is not Resident is late.
    Drawn,
    /// Held past its radius, where the camera may come to draw it: its mips
    /// are requested as a drawn one's are, and none is late
    /// [ref:ue-5.8-texture-streaming-hidden-prefetch].
    Ahead,
    /// A Far proxy held while its Chunk is drawn in its place: it needs no
    /// mip.
    Replaced,
}

/// One Placement's use of a Texture that streams: the sphere around its
/// Mesh's bounds, and the Texture's texels per metre of its surface at
/// mip 0.
struct TextureUse {
    texture: AssetId,
    centre: DVec3,
    radius: f64,
    texels_per_metre: f64,
}

/// The mips one Texture needs that are not Resident: the request that
/// brings them.
#[derive(Clone, Debug)]
pub(crate) struct MipRequest {
    pub texture: AssetId,
    pub levels: Range<u32>,
}

/// The loaded Chunks' uses of streamed Textures, and the requests in flight.
#[derive(Default)]
pub(crate) struct MipPriority {
    /// Each loaded Chunk's uses, by its Asset ID.
    uses: HashMap<AssetId, Vec<TextureUse>>,
    /// The finest mip each Texture needs, from where the camera will be a
    /// lead ahead, kept for its memory.
    needed: HashMap<AssetId, u32>,
    /// The finest mip each Texture's drawn uses need from where the camera
    /// is, kept for its memory.
    drawn_needed: HashMap<AssetId, u32>,
    /// The camera's position at the last `plan`.
    camera_before: Option<DVec3>,
    /// The `plan`s so far.
    plans: u64,
    /// The last `plan` that found each Texture needing its finest Resident
    /// mip or finer.
    last_used_plan: HashMap<AssetId, u64>,
    requests: Vec<MipRequest>,
    /// The Textures whose mips are requested and not yet applied.
    in_flight: HashSet<AssetId>,
    /// Textures whose read failed, which are not requested again until the
    /// Material pool frees an entry.
    refused: HashSet<AssetId>,
    /// The Material pool's frees the last `plan` saw.
    material_frees: u64,
}

impl MipPriority {
    /// The finest mip each Texture `chunks`' Placements sample needs, those
    /// drawn and held ahead, `chunks` being the loaded Chunks and the
    /// Resident Far proxies, each with its Sight, against what `materials`
    /// holds Resident: from `camera`, or, as the camera closes on a
    /// Placement at its rate since the last `plan`, from where it will be
    /// `LEAD_FRAMES` later, at most a Chunk edge nearer, as the Chunk
    /// streamer requests no further than one past its radius, unless it
    /// `teleported`. Returns the late mips,
    /// those the drawn Placements need from `camera` that are not Resident,
    /// and the requests for those not yet requested whose move finds room
    /// in its Channel, free or by an eviction, finest-needed first, then by
    /// Asset ID.
    pub(crate) fn plan<'chunks>(
        &mut self,
        camera: DVec3,
        teleported: bool,
        chunks: impl Iterator<Item = (&'chunks LoadedChunk, Sight)> + Clone,
        meshes: &LoadedMeshes,
        materials: &MaterialPool,
    ) -> (u32, &[MipRequest]) {
        if self.uses.len() > chunks.clone().count() {
            let loaded: HashSet<AssetId> = chunks.clone().map(|(chunk, _)| chunk.id).collect();
            self.uses.retain(|chunk, _| loaded.contains(chunk));
        }
        let material_frees = materials.generations().frees();
        if std::mem::replace(&mut self.material_frees, material_frees) != material_frees {
            self.refused.clear();
            self.last_used_plan
                .retain(|&texture, _| materials.is_texture_loaded(texture));
        }
        let camera_before = self.camera_before.replace(camera);
        let camera_before = camera_before.filter(|_| !teleported);
        self.needed.clear();
        self.drawn_needed.clear();
        self.plans += 1;
        for (chunk, sight) in chunks {
            let uses = self
                .uses
                .entry(chunk.id)
                .or_insert_with(|| texture_uses(chunk, meshes, materials));
            if sight == Sight::Replaced {
                continue;
            }
            for texture_use in uses.iter() {
                let distance_from = |position: DVec3| {
                    (position.distance(texture_use.centre) - texture_use.radius).max(0.0)
                };
                let distance = distance_from(camera);
                let closing = camera_before.map_or(0.0, |before| distance_from(before) - distance);
                let lead_metres = (closing.max(0.0) * LEAD_FRAMES).min(CHUNK_EDGE_METRES);
                let distance_ahead = (distance - lead_metres).max(0.0);
                let texels_per_metre = texture_use.texels_per_metre;
                let level = finest_mip_needed(texels_per_metre, distance_ahead);
                keep_finest(&mut self.needed, texture_use.texture, level);
                if sight == Sight::Drawn {
                    let level = finest_mip_needed(texels_per_metre, distance);
                    keep_finest(&mut self.drawn_needed, texture_use.texture, level);
                }
            }
        }
        for (&texture, &needed) in &self.needed {
            let resident = materials.texture_residency(texture);
            if resident.is_some_and(|(_, resident)| needed <= resident) {
                self.last_used_plan.insert(texture, self.plans);
            }
        }
        let mut late_mips = 0;
        for (&texture, &needed) in &self.drawn_needed {
            if let Some((_, resident)) = materials.texture_residency(texture) {
                late_mips += resident.saturating_sub(needed);
            }
        }
        self.requests.clear();
        let (needed_by_texture, last_used_plan) = (&self.needed, &self.last_used_plan);
        let eviction_rank = |texture, resident_level| {
            rank(
                needed_by_texture,
                last_used_plan,
                texture,
                resident_level,
                0,
            )
        };
        let mut evictable_needs = None;
        for (&texture, &needed) in needed_by_texture {
            let Some((_, resident)) = materials.texture_residency(texture) else {
                continue;
            };
            if needed >= resident {
                continue;
            }
            if self.in_flight.contains(&texture) || self.refused.contains(&texture) {
                continue;
            }
            if let Some(class) = materials.full_class_for_mips(texture, needed) {
                let needs = evictable_needs
                    .get_or_insert_with(|| materials.coarsest_evictable_needs(&eviction_rank));
                if needs.get(&class).is_none_or(|&coarsest| coarsest <= needed) {
                    continue;
                }
            }
            self.requests.push(MipRequest {
                texture,
                levels: needed..resident,
            });
        }
        self.requests
            .sort_unstable_by_key(|request| (request.levels.start, request.texture.0));
        for request in &self.requests {
            self.in_flight.insert(request.texture);
        }
        (late_mips, &self.requests)
    }

    /// How eviction ranks `texture`, whose finest Resident mip is
    /// `resident_level`, to make room for a move to `first_level`, by the
    /// last `plan`: `None` when it needs `first_level` or finer, or its
    /// finest Resident mip within one level, as it loaded at that need and
    /// so stays until its need coarsens by two (ADR 0017).
    pub(crate) fn eviction_rank(
        &self,
        texture: AssetId,
        resident_level: u32,
        first_level: u32,
    ) -> Option<EvictionRank> {
        rank(
            &self.needed,
            &self.last_used_plan,
            texture,
            resident_level,
            first_level,
        )
    }

    /// Whether any request is in flight.
    pub(crate) fn waiting(&self) -> bool {
        !self.in_flight.is_empty()
    }

    /// Ends `texture`'s request, `read_failed` when its mips were not read.
    pub(crate) fn finished(&mut self, texture: AssetId, read_failed: bool) {
        self.in_flight.remove(&texture);
        if read_failed {
            self.refused.insert(texture);
        }
    }
}

/// Keeps `level` as `texture`'s in `needed` where it is finer.
fn keep_finest(needed: &mut HashMap<AssetId, u32>, texture: AssetId, level: u32) {
    let finest = needed.entry(texture).or_insert(level);
    *finest = (*finest).min(level);
}

/// `MipPriority::eviction_rank` from its `needed` and `last_used_plan`.
fn rank(
    needed: &HashMap<AssetId, u32>,
    last_used_plan: &HashMap<AssetId, u64>,
    texture: AssetId,
    resident_level: u32,
    first_level: u32,
) -> Option<EvictionRank> {
    let finest_mip_needed = needed.get(&texture).copied().unwrap_or(u32::MAX);
    let evictable =
        finest_mip_needed > first_level && finest_mip_needed >= resident_level.saturating_add(2);
    evictable.then(|| EvictionRank {
        finest_mip_needed,
        last_used_plan: last_used_plan.get(&texture).copied().unwrap_or(0),
    })
}

/// The uses of streamed Textures by `chunk`'s Placements: each Primitive's
/// Material, or the Placement's own, and each Texture it samples that has
/// mips finer than those Resident.
fn texture_uses(
    chunk: &LoadedChunk,
    meshes: &LoadedMeshes,
    materials: &MaterialPool,
) -> Vec<TextureUse> {
    let mut uses = Vec::new();
    for placement in &chunk.placements {
        let Some(mesh) = meshes.get(placement.mesh) else {
            continue;
        };
        let mut sphere = None;
        for (&texel_density, &primitive_material) in
            mesh.texel_densities.iter().zip(&mesh.materials)
        {
            let Some(material) = placement.material.or(primitive_material) else {
                continue;
            };
            for &texture in materials.textures_of(material) {
                let Some((shape, resident_level)) = materials.texture_residency(texture) else {
                    continue;
                };
                if resident_level == 0 {
                    continue;
                }
                let (centre, radius, smallest_scale) = *sphere
                    .get_or_insert_with(|| placement_sphere(&placement.transform, mesh.bounds));
                let side = shape.width.max(shape.height);
                uses.push(TextureUse {
                    texture,
                    centre,
                    radius,
                    texels_per_metre: texels_per_metre(texel_density, side, smallest_scale),
                });
            }
        }
    }
    uses
}

/// The finest mip of a Texture `side` texels across at mip 0 that a Mesh
/// with `bounds`, placed by `transform` and sampling it at `texel_density`,
/// needs from `camera`, as `plan` judges a drawn use.
pub(crate) fn placement_finest_mip_needed(
    camera: DVec3,
    transform: &Transform,
    bounds: [[f32; 3]; 2],
    texel_density: f32,
    side: u32,
) -> u32 {
    let (centre, radius, smallest_scale) = placement_sphere(transform, bounds);
    let distance = (camera.distance(centre) - radius).max(0.0);
    finest_mip_needed(
        texels_per_metre(texel_density, side, smallest_scale),
        distance,
    )
}

/// The texels per metre of a surface, at mip 0, of a Texture `side` texels
/// across that a Primitive samples at `texel_density`, placed at
/// `smallest_scale`.
fn texels_per_metre(texel_density: f32, side: u32, smallest_scale: f64) -> f64 {
    f64::from(texel_density) * f64::from(side) / smallest_scale
}

/// The centre and radius of the sphere around `bounds`, a Mesh's, placed
/// by `transform`, and the transform's smallest scale.
fn placement_sphere(transform: &Transform, bounds: [[f32; 3]; 2]) -> (DVec3, f64, f64) {
    let [lowest, highest] = bounds.map(|corner| glam::Vec3::from(corner).as_dvec3());
    let scale = transform.scale.as_dvec3();
    let centre =
        transform.translation + transform.rotation.as_dquat() * (scale * (lowest + highest) / 2.0);
    let radius = scale.abs().max_element() * (highest - lowest).length() / 2.0;
    (centre, radius, scale.abs().min_element())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At 45° and 1440 pixels tall, a pixel at a 16:9 view's corner spans
    /// 0.0005753 m per metre on the axis, over 1.7138 off it: 1 texel per
    /// pixel is 2,979 texels per metre at 1 m, and each doubling of the
    /// distance coarsens the mip needed by one.
    #[test]
    fn a_surface_needs_the_mip_whose_texels_fit_the_finest_pixel_footprint() {
        let one_texel_per_pixel_at_one_metre = 1.7138 / 0.000_575_3;
        let texels_per_metre = 1024.0 * one_texel_per_pixel_at_one_metre;

        assert_eq!(finest_mip_needed(texels_per_metre, 0.0), 0);
        assert_eq!(finest_mip_needed(texels_per_metre, 1.0 / 1024.0 * 1.99), 0);
        assert_eq!(finest_mip_needed(texels_per_metre, 1.0 / 1024.0 * 2.01), 1);
        assert_eq!(finest_mip_needed(texels_per_metre, 0.99), 9);
        assert_eq!(finest_mip_needed(texels_per_metre, 1.01), 10);
        assert_eq!(finest_mip_needed(texels_per_metre, 2.01), 11);
    }

    /// A finest mip loaded at a need gives way once that need coarsens by
    /// two levels, not one, and only to a move finer than its need; a
    /// Texture nothing drawn samples ranks with the coarsest need.
    #[test]
    fn a_finest_mip_gives_way_once_its_need_coarsens_by_two() {
        let mut priority = MipPriority::default();
        let (seen, unseen) = (AssetId(1), AssetId(2));
        priority.needed.insert(seen, 3);
        priority.last_used_plan.insert(seen, 7);

        assert_eq!(priority.eviction_rank(seen, 2, 0), None);
        assert_eq!(
            priority.eviction_rank(seen, 1, 0),
            Some(EvictionRank {
                finest_mip_needed: 3,
                last_used_plan: 7
            })
        );
        assert_eq!(priority.eviction_rank(seen, 1, 3), None);
        assert_eq!(
            priority.eviction_rank(unseen, 0, 0),
            Some(EvictionRank {
                finest_mip_needed: u32::MAX,
                last_used_plan: 0
            })
        );
    }

    struct TailReader {
        texture: crate::cooked::CookedTexture,
    }

    impl crate::assets::AssetReader for TailReader {
        fn material(&self, id: AssetId) -> anyhow::Result<crate::cooked::CookedMaterial> {
            anyhow::bail!("no Material {id}")
        }
        fn texture(&self, _id: AssetId) -> anyhow::Result<crate::cooked::CookedTexture> {
            Ok(self.texture.clone())
        }
        fn mesh(&self, id: AssetId) -> anyhow::Result<crate::cooked::CookedMesh> {
            anyhow::bail!("no Mesh {id}")
        }
        fn chunk(&self, id: AssetId) -> anyhow::Result<crate::cooked::CookedChunk> {
            anyhow::bail!("no Chunk {id}")
        }
    }

    /// A Material pool holding `textures`, 2,048² BC7 Textures with only
    /// their Resident tails, from mip 3, Resident.
    fn pool_of_tails(
        context: &crate::gpu::GpuContext,
        textures: &[AssetId],
    ) -> (MaterialPool, crate::render::staging::LoadStaging) {
        let format = wgpu::TextureFormat::Bc7RgbaUnormSrgb;
        let level = |level| vec![0; crate::cooked::level_bytes(format, 2048, 2048, level)];
        let reader = TailReader {
            texture: crate::cooked::CookedTexture {
                format,
                width: 2048,
                height: 2048,
                first_level: 3,
                levels: (3..12).map(level).collect(),
            },
        };
        let mut staging = crate::render::staging::LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(
            &context.device,
            &crate::stats::gpu_memory::tests::gpu_memory(context),
            &mut staging,
            &crate::assets::PoolCapacities::checked_in(),
            false,
        );
        for &texture in textures {
            let channel = crate::assets::MaterialChannel::Albedo;
            pool.load_texture(
                &context.device,
                &mut staging,
                &reader,
                channel,
                texture,
                &mut |_| false,
            )
            .expect("a tail loads");
        }
        (pool, staging)
    }

    fn chunk(id: u128, held: bool) -> LoadedChunk {
        LoadedChunk {
            id: AssetId(id),
            coordinates: [0, 0],
            placements: Vec::new(),
            lights: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            held,
        }
    }

    /// One texel a pixel at 1 m: a use `distance` metres away needs mip
    /// ⌊log₂ distance⌋.
    fn texture_use(texture: AssetId, distance: f64) -> TextureUse {
        TextureUse {
            texture,
            centre: DVec3::new(0.0, 0.0, distance),
            radius: 0.0,
            texels_per_metre: 1.0 / finest_pixel_metres_per_metre(),
        }
    }

    /// Three Textures at their tails, mip 3, each needing mip 1: the drawn
    /// Chunk's is late by two mips and requested; the one of a Chunk held
    /// ahead is requested and not late; the one of a Far proxy whose Chunk
    /// is drawn in its place is neither.
    #[test]
    fn a_chunk_held_ahead_has_its_mips_requested_without_counting_them_late() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let [drawn, ahead, replaced] = [10, 11, 12].map(AssetId);
        let (pool, mut staging) = pool_of_tails(&context, &[drawn, ahead, replaced]);
        let chunks = [chunk(1, false), chunk(2, true), chunk(3, true)];
        let mut priority = MipPriority::default();
        for (chunk, texture) in chunks.iter().zip([drawn, ahead, replaced]) {
            priority
                .uses
                .insert(chunk.id, vec![texture_use(texture, 2.5)]);
        }
        let sights = [Sight::Drawn, Sight::Ahead, Sight::Replaced];

        let (late_mips, requests) = priority.plan(
            DVec3::ZERO,
            false,
            chunks.iter().zip(sights),
            &LoadedMeshes::default(),
            &pool,
        );

        let requested: Vec<_> = (requests.iter())
            .map(|request| (request.texture, request.levels.clone()))
            .collect();
        assert_eq!(late_mips, 2);
        assert_eq!(requested, [(drawn, 1..3), (ahead, 1..3)]);
        staging.submit(&context.queue);
    }

    /// A drawn Texture at its tail, mip 3, 8.5 m away, then 8.2 m: closing
    /// 0.3 m a frame, it is 3.4 m away `LEAD_FRAMES` later, so mip 1 is
    /// requested while mip 3 still serves it and none is late; after a
    /// teleport to the same place nothing is requested. Closing 18.6 m in a
    /// frame to 300 m away, it would be 2.4 m away `LEAD_FRAMES` later, but
    /// the lead reaches no further than a Chunk edge, 256 m, to 44 m, where
    /// mip 5 serves it, so nothing is requested.
    #[test]
    fn a_surface_the_camera_closes_on_has_its_mips_requested_a_lead_ahead() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let texture = AssetId(10);
        let (pool, mut staging) = pool_of_tails(&context, &[texture]);
        let chunks = [chunk(1, false)];
        let plan_from = |first_z: f64, second_z: f64, teleported: bool| {
            let mut priority = MipPriority::default();
            priority
                .uses
                .insert(AssetId(1), vec![texture_use(texture, 0.0)]);
            let mut plan = |camera_z: f64, teleported: bool| {
                let (late_mips, requests) = priority.plan(
                    DVec3::new(0.0, 0.0, camera_z),
                    teleported,
                    chunks.iter().zip([Sight::Drawn]),
                    &LoadedMeshes::default(),
                    &pool,
                );
                let requested: Vec<_> = (requests.iter())
                    .map(|request| (request.texture, request.levels.clone()))
                    .collect();
                (late_mips, requested)
            };
            assert_eq!(plan(first_z, false), (0, vec![]));
            plan(second_z, teleported)
        };

        assert_eq!((LEAD_FRAMES, CHUNK_EDGE_METRES), (16.0, 256.0));
        assert_eq!(plan_from(-8.5, -8.2, false), (0, vec![(texture, 1..3)]));
        assert_eq!(plan_from(-8.5, -8.2, true), (0, vec![]));
        assert_eq!(plan_from(-318.6, -300.0, false), (0, vec![]));
        staging.submit(&context.queue);
    }
}
