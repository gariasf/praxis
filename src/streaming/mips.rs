//! Mip priority (ADR 0017): the finest mip each streamed Texture needs, the
//! finest any loaded Chunk's Placement sampling it needs from its distance
//! and its Primitive's UV texel density against the 1440p pixel footprint,
//! and the mips to request from the IO pool for it, finest-needed first.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use glam::DVec3;

use crate::assets::material::EvictionRank;
use crate::assets::{LoadedMeshes, MaterialPool};
use crate::camera::FIELD_OF_VIEW;
use crate::components::Transform;
use crate::cooked::AssetId;
use crate::streaming::LoadedChunk;

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
    /// The finest mip each Texture needs this frame, kept for its memory.
    needed: HashMap<AssetId, u32>,
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
    /// The finest mip each Texture `chunks`' drawn Placements sample needs
    /// from `camera`, against what `materials` holds Resident. Returns the
    /// late mips, those needed and not Resident, and the requests for those
    /// not yet requested whose move finds room in its Channel, free or by an
    /// eviction, finest-needed first, then by Asset ID.
    pub(crate) fn plan(
        &mut self,
        camera: DVec3,
        chunks: &[LoadedChunk],
        meshes: &LoadedMeshes,
        materials: &MaterialPool,
    ) -> (u32, &[MipRequest]) {
        if self.uses.len() > chunks.len() {
            let loaded: HashSet<AssetId> = chunks.iter().map(|chunk| chunk.id).collect();
            self.uses.retain(|chunk, _| loaded.contains(chunk));
        }
        let material_frees = materials.generations().frees();
        if std::mem::replace(&mut self.material_frees, material_frees) != material_frees {
            self.refused.clear();
            self.last_used_plan
                .retain(|&texture, _| materials.is_texture_loaded(texture));
        }
        self.needed.clear();
        self.plans += 1;
        for chunk in chunks {
            let uses = self
                .uses
                .entry(chunk.id)
                .or_insert_with(|| texture_uses(chunk, meshes, materials));
            if chunk.held {
                continue;
            }
            for texture_use in uses.iter() {
                let distance = (camera.distance(texture_use.centre) - texture_use.radius).max(0.0);
                let level = finest_mip_needed(texture_use.texels_per_metre, distance);
                let needed = self.needed.entry(texture_use.texture).or_insert(level);
                *needed = (*needed).min(level);
            }
        }
        for (&texture, &needed) in &self.needed {
            let resident = materials.texture_residency(texture);
            if resident.is_some_and(|(_, resident)| needed <= resident) {
                self.last_used_plan.insert(texture, self.plans);
            }
        }
        let mut late_mips = 0;
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
            late_mips += resident - needed;
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
                    texels_per_metre: f64::from(texel_density) * f64::from(side) / smallest_scale,
                });
            }
        }
    }
    uses
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
}
