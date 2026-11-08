use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Instant;

use anyhow::Context;

use crate::assets::loader::AssetReader;
use crate::assets::pools::{ENTRIES_FREED_PER_FRAME, Generations, PoolCapacities, Unused};
use crate::cooked::{AssetId, CookedTexture, resident_tail_level};
use crate::gpu::{FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE, FLOOR_TEXTURE_ARRAY_LAYERS};
use crate::render::LoadStaging;
use crate::render::bind_groups::MATERIALS;
use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// Texture classes each Channel's binding array can hold: the Channels split
/// the Capability floor's binding-array elements per stage (ADR 0021),
/// 13,107 each.
pub(crate) const CLASSES_PER_CHANNEL: u32 =
    FLOOR_BINDING_ARRAY_ELEMENTS_PER_STAGE / MaterialChannel::ALL.len() as u32;

/// The most slices a Texture class holds, the floor's array layers, and the
/// Layers each class spans: a Layer is its class times this plus its slice.
pub(crate) const SLICES_PER_CLASS: u32 = FLOOR_TEXTURE_ARRAY_LAYERS;

/// The colour Channels' anisotropy (albedo, emissive) and the data
/// Channels' (normal, metallic-roughness, occlusion), the owner's pick from
/// the candidates measured in `docs/performance.md`: 4x on the data
/// Channels shortens minification's lit pass and drops the sparkle
/// of highlights whose normals came from finer mips than Toksvig widened
/// their roughness for.
const COLOUR_ANISOTROPY: u16 = 16;
const DATA_ANISOTROPY: u16 = 4;

/// A Material's index in the Material SSBO, which the GPU sees, and the
/// generation of that index it was loaded in, which no later Material there
/// shares.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MaterialHandle {
    pub index: u32,
    pub generation: u32,
}

impl MaterialHandle {
    /// Index into the material SSBO.
    pub fn index(self) -> u32 {
        self.index
    }
}

/// One texture channel of a material. Each channel has its own binding array;
/// a material stores one Layer per channel. The discriminant is the
/// Channel's position in `MaterialPool::channels` and part of every
/// Texture's Asset ID: never renumber.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MaterialChannel {
    Albedo = 0,
    Normal = 1,
    MetallicRoughness = 2,
    Occlusion = 3,
    Emissive = 4,
}

impl MaterialChannel {
    pub const ALL: [Self; 5] = [
        Self::Albedo,
        Self::Normal,
        Self::MetallicRoughness,
        Self::Occlusion,
        Self::Emissive,
    ];
}

/// One entry in the material SSBO. All-`vec4` layout: WGSL pads `vec3<f32>` to
/// 16 bytes silently, so every field is 16-byte sized to match the WGSL struct.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub metallic_roughness: [f32; 4], // x=metallic, y=roughness, z=normal scale, w=pad
    pub emissive: [f32; 4],           // xyz=color, w=pad (unread)
    pub layers: [u32; 4],             // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],              // x=emissive Layer, yzw=pad
}

/// What every Texture of one Texture class shares.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextureShape {
    pub width: u32,
    pub height: u32,
    pub mip_level_count: u32,
    pub format: wgpu::TextureFormat,
}

impl TextureShape {
    pub(crate) fn of(texture: &CookedTexture) -> Self {
        Self {
            width: texture.width,
            height: texture.height,
            mip_level_count: texture.first_level + texture.levels.len() as u32,
            format: texture.format,
        }
    }

    /// The first mip of its Resident tail (`cooked::resident_tail_level`).
    fn resident_tail_level(self) -> u32 {
        resident_tail_level(self.format, self.width, self.height, self.mip_level_count)
    }

    /// The shape of its mips from `level` on.
    fn mips_from(self, level: u32) -> Self {
        Self {
            width: self.width >> level,
            height: self.height >> level,
            mip_level_count: self.mip_level_count - level,
            format: self.format,
        }
    }
}

/// A Texture's place in its Channel: its Texture class, which indexes the
/// Channel's binding array, and its slice of that class's texture array.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassAndSlice {
    pub class: u32,
    pub slice: u32,
}

impl ClassAndSlice {
    /// The Layer that names it, which WGSL's `layer_class_and_slice` decodes.
    pub(crate) fn layer(self) -> u32 {
        self.class * SLICES_PER_CLASS + self.slice
    }
}

/// One Channel's Texture classes, in the order they opened, up to its
/// capacity.
pub(crate) struct ClassTable {
    classes: Vec<ClassSlices>,
    capacity: u32,
}

/// A Texture class's shape, its slice count, fixed when it opens, the
/// slices taken from the front, and those released since, which `place`
/// gives out again first.
struct ClassSlices {
    shape: TextureShape,
    slices: u32,
    taken: u32,
    released: Vec<u32>,
}

impl ClassTable {
    pub(crate) fn new(capacity: u32) -> Self {
        Self {
            classes: Vec::new(),
            capacity,
        }
    }

    /// A released slice of a class of `shape`, else the next slice of the
    /// newest class of `shape`, or slice 0 of a new class when that one is
    /// full or there is none: the first of a shape has one slice, each later
    /// one twice the one before's, up to `SLICES_PER_CLASS`. `None` when the
    /// Channel holds its capacity of classes and none of `shape` has room.
    pub(crate) fn place(&mut self, shape: TextureShape) -> Option<ClassAndSlice> {
        let released = self
            .classes
            .iter_mut()
            .enumerate()
            .find_map(|(class, slices)| {
                let slice = (slices.shape == shape).then(|| slices.released.pop())??;
                Some(ClassAndSlice {
                    class: class as u32,
                    slice,
                })
            });
        if released.is_some() {
            return released;
        }
        let newest = self.classes.iter().rposition(|class| class.shape == shape);
        if let Some(class) = newest
            && let newest_class = &mut self.classes[class]
            && newest_class.taken < newest_class.slices
        {
            newest_class.taken += 1;
            return Some(ClassAndSlice {
                class: class as u32,
                slice: newest_class.taken - 1,
            });
        }
        if self.classes.len() as u32 >= self.capacity {
            return None;
        }
        let slices = newest.map_or(1, |class| {
            (self.classes[class].slices * 2).min(SLICES_PER_CLASS)
        });
        self.classes.push(ClassSlices {
            shape,
            slices,
            taken: 1,
            released: Vec::new(),
        });
        Some(ClassAndSlice {
            class: self.classes.len() as u32 - 1,
            slice: 0,
        })
    }

    /// The slices `class` was opened with.
    fn slices(&self, class: u32) -> u32 {
        self.classes[class as usize].slices
    }

    /// Gives `placement`'s slice back to its class, for `place` to give out
    /// again.
    pub(crate) fn release(&mut self, placement: ClassAndSlice) {
        self.classes[placement.class as usize]
            .released
            .push(placement.slice);
    }

    /// Whether `place` would find room for each of `shapes`, placed in turn.
    fn has_room(&self, shapes: &[TextureShape]) -> bool {
        let new_classes = shapes
            .iter()
            .filter(|&&shape| {
                let mut classes = self.classes.iter().filter(|class| class.shape == shape);
                classes.all(|class| class.released.is_empty())
                    && self
                        .classes
                        .iter()
                        .rfind(|class| class.shape == shape)
                        .is_none_or(|newest| newest.taken == newest.slices)
            })
            .count();
        self.classes.len() + new_classes <= self.capacity as usize
    }
}

/// Where a loaded Texture lies in its Channel: its Resident tail's slice,
/// and its chain's, its mips from its finest Resident one, when it has finer
/// mips there, whose Layer it is sampled at.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct TexturePlacement {
    tail: ClassAndSlice,
    chain: Option<ClassAndSlice>,
}

impl TexturePlacement {
    fn layer(self) -> u32 {
        self.chain.unwrap_or(self.tail).layer()
    }
}

/// A Texture loaded by Asset ID: its Channel, its shape, where it lies, its
/// finest Resident mip, and how many Materials and spawns loaded it, less
/// the freed Materials among them.
struct LoadedTexture {
    channel: MaterialChannel,
    shape: TextureShape,
    placement: TexturePlacement,
    resident_level: u32,
    users: u32,
}

/// What `MaterialPool::stream_mips` did with a Texture's mips.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum StreamedMips {
    /// The Texture moved to the class of its new finest mip.
    Moved,
    /// Its Channel had no room for that class.
    Refused,
    /// The Texture is gone, or its finest Resident mip is no longer the one
    /// they end before.
    Stale,
}

/// A Material loaded by Asset ID: its Handle, how many loads returned it,
/// less those `remove_user` took back, when its last user left, and the
/// Textures it counts as a user of.
struct LoadedMaterial {
    handle: MaterialHandle,
    users: u32,
    unused_since: Option<u64>,
    textures: Vec<AssetId>,
}

/// One Channel's Textures, a texture array per Texture class, bound as one
/// partially bound binding array (ADR 0017).
struct ChannelTextures {
    /// Its binding's name in `MATERIALS`, which labels its arrays too.
    label: &'static str,
    memory: GpuMemory,
    table: ClassTable,
    /// Each Texture class's array and its view, at the class's index.
    textures: Vec<Counted<wgpu::Texture>>,
    views: Vec<wgpu::TextureView>,
}

impl ChannelTextures {
    fn new(label: &'static str, memory: &GpuMemory, classes: u32) -> Self {
        Self {
            label,
            memory: memory.clone(),
            table: ClassTable::new(classes),
            textures: Vec::new(),
            views: Vec::new(),
        }
    }

    /// Writes `texture` through `staging`: its Resident tail into a free
    /// slice of the tail's own Texture class, then, unless
    /// `holds_back_finer_mips`, the finer mips it holds into a free slice of
    /// the class of its mips from its finest, beside the tail copied over
    /// from the tail's slice. Returns where it lies, without a chain when it
    /// holds no finer mip or the Channel has no room for one, and its finest
    /// Resident mip; or `None` when the Channel has no room for its tail.
    fn push(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        texture: &CookedTexture,
        holds_back_finer_mips: bool,
    ) -> Option<(TexturePlacement, u32)> {
        let shape = TextureShape::of(texture);
        let tail_level = shape.resident_tail_level();
        let finer_count = tail_level
            .checked_sub(texture.first_level)
            .expect("a Texture holds its whole Resident tail");
        let (finer_levels, tail_levels) = texture.levels.split_at(finer_count as usize);
        let tail = self.place(device, shape.mips_from(tail_level))?;
        for (level, bytes) in tail_levels.iter().enumerate() {
            let array = &self.textures[tail.class as usize];
            staging.write_texture(array, level as u32, tail.slice, bytes);
        }
        let tail_only = (TexturePlacement { tail, chain: None }, tail_level);
        if finer_levels.is_empty() || holds_back_finer_mips {
            return Some(tail_only);
        }
        let Some(chain) = self.place(device, shape.mips_from(texture.first_level)) else {
            return Some(tail_only);
        };
        let chain_array = &self.textures[chain.class as usize];
        for (level, bytes) in finer_levels.iter().enumerate() {
            staging.write_texture(chain_array, level as u32, chain.slice, bytes);
        }
        let tail_array = &self.textures[tail.class as usize];
        staging.copy_texture(
            tail_array,
            tail.slice,
            chain_array,
            chain.slice,
            finer_count,
        );
        let placement = TexturePlacement {
            tail,
            chain: Some(chain),
        };
        Some((placement, texture.first_level))
    }

    /// Whether `push` would find room for `texture`'s Resident tail and,
    /// unless `holds_back_finer_mips`, the finer mips it holds.
    fn has_room(&self, texture: &CookedTexture, holds_back_finer_mips: bool) -> bool {
        let shape = TextureShape::of(texture);
        let tail_level = shape.resident_tail_level();
        let tail = shape.mips_from(tail_level);
        if texture.first_level >= tail_level || holds_back_finer_mips {
            self.table.has_room(&[tail])
        } else {
            self.table
                .has_room(&[tail, shape.mips_from(texture.first_level)])
        }
    }

    /// Gives back the slices `placement` holds.
    fn release(&mut self, placement: TexturePlacement) {
        self.table.release(placement.tail);
        if let Some(chain) = placement.chain {
            self.table.release(chain);
        }
    }

    /// A free slice of `shape`'s Texture class, its array created when the
    /// class opens, or `None`, logged, when the Channel has no room for it.
    fn place(&mut self, device: &wgpu::Device, shape: TextureShape) -> Option<ClassAndSlice> {
        let Some(placement) = self.table.place(shape) else {
            tracing::error!(
                channel = self.label,
                capacity = self.table.capacity,
                "channel has no room for another Texture class; the Texture keeps its Resident tail, or Layer 0 without one"
            );
            return None;
        };
        if placement.slice == 0 {
            let array = self.memory.create_material_texture(
                device,
                &wgpu::TextureDescriptor {
                    label: Some(self.label),
                    size: wgpu::Extent3d {
                        width: shape.width,
                        height: shape.height,
                        depth_or_array_layers: self.table.slices(placement.class),
                    },
                    mip_level_count: shape.mip_level_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: shape.format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                },
            );
            // An array of one slice would otherwise get a 2D view.
            self.views
                .push(array.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                }));
            self.textures.push(array);
        }
        Some(placement)
    }
}

/// Material-as-ID: one SSBO of `MaterialData` indexed by `MaterialHandle`, plus
/// one binding array of Texture classes per Channel. A single bind group
/// serves every material; the fragment shader looks up the Material ID its
/// Meshlet record carries, the Instance's override or else the Meshlet's own,
/// and samples each Channel at the class and slice of the Layer stored in
/// `layers`.
pub struct MaterialPool {
    generations: Generations,
    /// Each Material's data, at its Handle's index; a freed one's stays
    /// until a later Material takes its index.
    materials: Vec<MaterialData>,
    /// The Asset ID `load` loaded each Material from, at its Handle's index.
    material_ids: Vec<Option<AssetId>>,
    materials_buffer: Counted<wgpu::Buffer>,
    channels: [ChannelTextures; 5],
    /// Each loaded Texture, which its Asset ID names.
    loaded_textures: HashMap<AssetId, LoadedTexture>,
    /// Each loaded Material, which its Asset ID names.
    loaded_materials: HashMap<AssetId, LoadedMaterial>,
    /// The Materials no load uses any more, which stay loaded until a
    /// refused load frees them.
    unused: Unused<AssetId>,
    /// Materials and Textures freed to make room since `take_frame_frees`;
    /// frees to make room stop once it reaches `ENTRIES_FREED_PER_FRAME`.
    frame_frees: usize,
    colour_sampler: wgpu::Sampler,
    data_sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    /// Set when a load opens a Texture class the bind group lacks.
    bind_group_is_stale: bool,
    /// The residency switch: every Texture loads its Resident tail alone.
    holds_back_finer_mips: bool,
    material_capacity: u32,
}

impl MaterialPool {
    /// Layer 0 of every Channel holds a 1×1 neutral texture: white for
    /// albedo / metallic-roughness / occlusion / emissive, the flat normal,
    /// x and y at 0.5, for normal. A material without a channel points here,
    /// samples a no-op, and its factors pass through unchanged.
    pub const DEFAULT_LAYER: u32 = 0;

    /// The normal Channel's Layer 0 texel, `Rgba16Float`: 0.5, 0.5, 1 and 1
    /// as binary16. 0.5 has no `Unorm8` code (128 decodes to 0.502, a tilt
    /// of 0.32°), so the shader's `encoded * 2.0 - 1.0` gives exactly 0 only
    /// from a float format.
    const FLAT_NORMAL_TEXEL: [u16; 4] = [0x3800, 0x3800, 0x3C00, 0x3C00];

    /// The neutral default `channel` holds in Layer 0 (see DEFAULT_LAYER):
    /// colour Channels are sRGB, data Channels linear.
    pub(crate) fn neutral_texture(channel: MaterialChannel) -> CookedTexture {
        let (format, texel) = match channel {
            MaterialChannel::Albedo | MaterialChannel::Emissive => {
                (wgpu::TextureFormat::Rgba8UnormSrgb, vec![255; 4])
            }
            MaterialChannel::Normal => (
                wgpu::TextureFormat::Rgba16Float,
                Self::FLAT_NORMAL_TEXEL
                    .iter()
                    .flat_map(|half| half.to_le_bytes())
                    .collect(),
            ),
            MaterialChannel::MetallicRoughness | MaterialChannel::Occlusion => {
                (wgpu::TextureFormat::Rgba8Unorm, vec![255; 4])
            }
        };
        CookedTexture {
            format,
            width: 1,
            height: 1,
            first_level: 0,
            levels: vec![texel],
        }
    }

    /// A pool holding each Channel's Layer 0, whose Textures load their
    /// Resident tails alone when `holds_back_finer_mips`.
    pub fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        staging: &mut LoadStaging,
        capacities: &PoolCapacities,
        holds_back_finer_mips: bool,
    ) -> Self {
        let materials_buffer = memory.create_buffer(
            device,
            GpuMemoryKind::SceneAndPassBuffers,
            &wgpu::BufferDescriptor {
                label: Some("materials_buffer"),
                size: u64::from(capacities.materials) * std::mem::size_of::<MaterialData>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        // Ordered as `MaterialChannel`.
        let mut channels = [
            "albedo_textures",
            "normal_textures",
            "metallic_roughness_textures",
            "occlusion_textures",
            "emissive_textures",
        ]
        .map(|label| ChannelTextures::new(label, memory, capacities.texture_classes_per_channel));

        for channel in MaterialChannel::ALL {
            let neutral = Self::neutral_texture(channel);
            channels[channel as usize].push(device, staging, &neutral, holds_back_finer_mips);
        }

        let colour_sampler = trilinear_sampler(device, "Colour Channel Sampler", COLOUR_ANISOTROPY);
        let data_sampler = trilinear_sampler(device, "Data Channel Sampler", DATA_ANISOTROPY);

        let bind_group_layout = MATERIALS.layout(device);
        let bind_group = create_bind_group(
            device,
            &bind_group_layout,
            &materials_buffer,
            &channels,
            &colour_sampler,
            &data_sampler,
        );

        Self {
            generations: Generations::default(),
            materials: Vec::new(),
            material_ids: Vec::new(),
            materials_buffer,
            channels,
            loaded_textures: HashMap::new(),
            loaded_materials: HashMap::new(),
            unused: Unused::default(),
            frame_frees: 0,
            colour_sampler,
            data_sampler,
            bind_group_layout,
            bind_group,
            bind_group_is_stale: false,
            holds_back_finer_mips,
            material_capacity: capacities.materials,
        }
    }

    /// Registers a material in the SSBO through `staging`, returning its
    /// handle, or `None`, logged, when the SSBO is full.
    pub fn insert(
        &mut self,
        staging: &mut LoadStaging,
        data: MaterialData,
    ) -> Option<MaterialHandle> {
        if self.refused_as_full() {
            return None;
        }
        let (index, generation) = self.generations.take();
        let offset = u64::from(index) * std::mem::size_of::<MaterialData>() as u64;
        staging.write(&self.materials_buffer, offset, bytemuck::bytes_of(&data));
        if index as usize == self.materials.len() {
            self.materials.push(data);
            self.material_ids.push(None);
        } else {
            self.materials[index as usize] = data;
            self.material_ids[index as usize] = None;
        }
        tracing::debug!(handle = index, "material registered");
        Some(MaterialHandle { index, generation })
    }

    /// Whether the SSBO is full, logged when it is.
    fn refused_as_full(&self) -> bool {
        let full = self.generations.full(self.material_capacity);
        if full {
            tracing::warn!(
                capacity = self.material_capacity,
                "Material refused: the Material SSBO is full"
            );
        }
        full
    }

    /// The data of `handle`'s Material, or None when it was freed.
    #[cfg_attr(not(all(test, feature = "cook")), allow(dead_code))]
    pub fn get(&self, handle: MaterialHandle) -> Option<&MaterialData> {
        self.generations
            .live(handle.index, handle.generation)
            .then(|| &self.materials[handle.index as usize])
    }

    /// Releases `handle`'s index, which the next Material inserted takes, so
    /// `handle` finds nothing again, and gives back the slices of its
    /// Textures that no other loaded Material samples. A Mesh whose
    /// Primitive names it must be freed first. A Handle that finds no
    /// Material is warned about.
    #[cfg_attr(not(all(test, feature = "cook")), allow(dead_code))]
    pub fn free(&mut self, handle: MaterialHandle) {
        if !self.generations.free(handle.index, handle.generation) {
            tracing::warn!(?handle, "Material not freed: its Handle finds no Material");
            return;
        }
        let freed_id = self
            .loaded_materials
            .iter()
            .find_map(|(&id, loaded)| (loaded.handle == handle).then_some(id));
        if let Some(id) = freed_id
            && let Some(loaded) = self.loaded_materials.remove(&id)
        {
            if let Some(since) = loaded.unused_since {
                self.unused.remove(since);
            }
            self.release_textures(&loaded.textures);
        }
        tracing::debug!(?handle, "Material freed");
    }

    /// Frees the least recently used unused Material as `free` does, and
    /// tells `reader` which Assets it freed; false when none is unused or the
    /// frame freed `ENTRIES_FREED_PER_FRAME` entries already.
    pub fn free_least_recently_used(&mut self, reader: &dyn AssetReader) -> bool {
        if self.frame_frees >= ENTRIES_FREED_PER_FRAME {
            return false;
        }
        let Some(id) = self.unused.pop_least_recent() else {
            return false;
        };
        let loaded = self
            .loaded_materials
            .remove(&id)
            .expect("an unused Material is loaded");
        let handle = loaded.handle;
        self.generations.free(handle.index, handle.generation);
        let mut freed = self.release_textures(&loaded.textures);
        freed.push(id);
        self.frame_frees += freed.len();
        reader.forget(&freed);
        tracing::debug!(?handle, "Material freed");
        true
    }

    /// Frees the least recently used unused Material, or else has
    /// `make_room` free what keeps Materials in use; false when neither
    /// freed anything.
    fn free_for_room(
        &mut self,
        reader: &dyn AssetReader,
        make_room: &mut dyn FnMut(&mut MaterialPool) -> bool,
    ) -> bool {
        self.free_least_recently_used(reader) || make_room(self)
    }

    /// Counts a user fewer of each of `textures`, giving back the slices of
    /// those left with none, whose Asset IDs it returns.
    fn release_textures(&mut self, textures: &[AssetId]) -> Vec<AssetId> {
        let mut freed = Vec::new();
        for &id in textures {
            let Entry::Occupied(mut entry) = self.loaded_textures.entry(id) else {
                continue;
            };
            entry.get_mut().users -= 1;
            if entry.get().users == 0 {
                let texture = entry.remove();
                self.channels[texture.channel as usize].release(texture.placement);
                freed.push(id);
            }
        }
        freed
    }

    /// Materials and Textures freed to make room since the last call, which
    /// renews the frame's `ENTRIES_FREED_PER_FRAME`.
    pub fn take_frame_frees(&mut self) -> usize {
        std::mem::take(&mut self.frame_frees)
    }

    /// Which Material Handles find a Material, and how many Materials were
    /// freed, as prepare reads them.
    pub(crate) fn generations(&self) -> &Generations {
        &self.generations
    }

    /// Counts one user fewer of the Material `id`; one left with none stays
    /// loaded, the most recently used of the unused Materials.
    pub fn remove_user(&mut self, id: AssetId) {
        let Some(loaded) = self.loaded_materials.get_mut(&id) else {
            return;
        };
        if loaded.users == 0 {
            return;
        }
        loaded.users -= 1;
        if loaded.users == 0 {
            loaded.unused_since = Some(self.unused.push(id));
        }
    }

    /// How many loads returned the Material `id`, less those `remove_user`
    /// took back: a user per loaded Chunk whose Placements name it, per
    /// Primitive of a loaded Mesh that names it, and per spawn that loaded
    /// it.
    #[cfg_attr(not(all(test, feature = "cook")), allow(dead_code))]
    pub fn users(&self, id: AssetId) -> u32 {
        self.loaded_materials
            .get(&id)
            .map_or(0, |loaded| loaded.users)
    }

    /// `load_making_room`, with no room made but by freeing unused Materials.
    #[cfg(all(test, feature = "cook"))]
    pub fn load(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        reader: &dyn AssetReader,
        id: AssetId,
    ) -> anyhow::Result<MaterialHandle> {
        self.load_making_room(device, staging, reader, id, &mut |_| false)
    }

    /// Loads the Material `id` from `reader`, and each Texture it samples, once
    /// per Asset ID, written through `staging`, and returns its Handle, or
    /// the SSBO's refusal when it is full; a load of a loaded Material counts
    /// one more user of it. A full SSBO frees unused Materials, least
    /// recently used first, to make room, and once none is left has
    /// `make_room` free what keeps Materials in use. A Channel the Material
    /// lacks samples Layer 0, as does one already full.
    pub fn load_making_room(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        reader: &dyn AssetReader,
        id: AssetId,
        make_room: &mut dyn FnMut(&mut MaterialPool) -> bool,
    ) -> anyhow::Result<MaterialHandle> {
        if let Some(loaded) = self.loaded_materials.get_mut(&id) {
            loaded.users += 1;
            if let Some(since) = loaded.unused_since.take() {
                self.unused.remove(since);
            }
            return Ok(loaded.handle);
        }
        while self.generations.full(self.material_capacity) && self.free_for_room(reader, make_room)
        {
        }
        if self.refused_as_full() {
            anyhow::bail!("the Material pool refused Material {id}");
        }
        let material = reader.material(id)?;
        let mut layers = [Self::DEFAULT_LAYER; 5];
        let mut textures = Vec::new();
        for (channel, texture_id) in MaterialChannel::ALL.into_iter().zip(material.textures) {
            if let Some(texture_id) = texture_id {
                let loaded =
                    self.load_texture(device, staging, reader, channel, texture_id, make_room);
                layers[channel as usize] = match loaded {
                    Ok(layer) => layer,
                    Err(error) => {
                        let freed = self.release_textures(&textures);
                        reader.forget(&freed);
                        return Err(error.context(format!("loading Material {id}")));
                    }
                };
                if self.loaded_textures.contains_key(&texture_id) {
                    textures.push(texture_id);
                }
            }
        }
        let [red, green, blue] = material.emissive;
        let data = MaterialData {
            base_color: material.base_color,
            metallic_roughness: [
                material.metallic,
                material.roughness,
                material.normal_scale,
                0.0,
            ],
            emissive: [red, green, blue, 1.0],
            layers: [layers[0], layers[1], layers[2], layers[3]],
            extra: [layers[4], 0, 0, 0],
        };
        let handle = self
            .insert(staging, data)
            .with_context(|| format!("the Material pool refused Material {id}"))?;
        let loaded = LoadedMaterial {
            handle,
            users: 1,
            unused_since: None,
            textures,
        };
        self.loaded_materials.insert(id, loaded);
        self.material_ids[handle.index as usize] = Some(id);
        tracing::debug!(material = %id, ?layers, "Material loaded");
        Ok(handle)
    }

    /// The bytes `load` writes through its staging for a Material, its
    /// Textures' aside.
    pub const WRITTEN_BYTES: u64 = size_of::<MaterialData>() as u64;

    /// Whether `load` loaded the Material `id`.
    pub fn is_loaded(&self, id: AssetId) -> bool {
        self.loaded_materials.contains_key(&id)
    }

    /// Whether `load_texture` placed the Texture `id`.
    pub fn is_texture_loaded(&self, id: AssetId) -> bool {
        self.loaded_textures.contains_key(&id)
    }

    /// Loads the Texture `id` from `reader` into `channel` once per Asset ID,
    /// written through `staging`, counting one more user of it, and returns
    /// its Layer: a free slice of the class of its mips from the finest
    /// `reader` gives, or of its Resident tail's when `reader` gives its tail
    /// alone, the pool holds back finer mips or the Channel has no room for
    /// its chain, or Layer 0 when it has none for its tail either. A Channel
    /// without room frees as `load` does until it has.
    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        reader: &dyn AssetReader,
        channel: MaterialChannel,
        id: AssetId,
        make_room: &mut dyn FnMut(&mut MaterialPool) -> bool,
    ) -> anyhow::Result<u32> {
        if let Some(loaded) = self.loaded_textures.get_mut(&id) {
            loaded.users += 1;
            return Ok(loaded.placement.layer());
        }
        let texture = reader.texture(id)?;
        let holds_back_finer_mips = self.holds_back_finer_mips;
        while !self.channels[channel as usize].has_room(&texture, holds_back_finer_mips)
            && self.free_for_room(reader, make_room)
        {}
        let channel_textures = &mut self.channels[channel as usize];
        let classes_before = channel_textures.views.len();
        let Some((placement, resident_level)) =
            channel_textures.push(device, staging, &texture, holds_back_finer_mips)
        else {
            return Ok(Self::DEFAULT_LAYER);
        };
        self.bind_group_is_stale |= channel_textures.views.len() != classes_before;
        let loaded = LoadedTexture {
            channel,
            shape: TextureShape::of(&texture),
            placement,
            resident_level,
            users: 1,
        };
        self.loaded_textures.insert(id, loaded);
        Ok(placement.layer())
    }

    /// Whether Textures stream their mips finer than their Resident tails,
    /// rather than the residency switch holding them back.
    pub(crate) fn streams_mips(&self) -> bool {
        !self.holds_back_finer_mips
    }

    /// The Textures the Material `handle` loaded with, none when it finds no
    /// Material `load` loaded.
    pub(crate) fn textures_of(&self, handle: MaterialHandle) -> &[AssetId] {
        if !self.generations.live(handle.index, handle.generation) {
            return &[];
        }
        self.material_ids[handle.index as usize]
            .and_then(|id| self.loaded_materials.get(&id))
            .map_or(&[], |loaded| &loaded.textures)
    }

    /// The loaded Texture `id`'s shape and its finest Resident mip.
    pub(crate) fn texture_residency(&self, id: AssetId) -> Option<(TextureShape, u32)> {
        let loaded = self.loaded_textures.get(&id)?;
        Some((loaded.shape, loaded.resident_level))
    }

    /// Moves the Texture `id` to the class of its mips from `first_level`
    /// through `staging`: `levels`, its mips from `first_level` to its finest
    /// Resident one, are written there, and its Resident mips copied after
    /// them from where it lies. Its Resident tail's slice stays taken, its
    /// earlier chain's is given back, and each loaded Material that samples
    /// it is written again with its new Layer.
    pub(crate) fn stream_mips(
        &mut self,
        device: &wgpu::Device,
        staging: &mut LoadStaging,
        id: AssetId,
        first_level: u32,
        levels: &[Vec<u8>],
    ) -> StreamedMips {
        let Some(loaded) = self.loaded_textures.get_mut(&id) else {
            return StreamedMips::Stale;
        };
        if first_level + levels.len() as u32 != loaded.resident_level {
            return StreamedMips::Stale;
        }
        let channel = &mut self.channels[loaded.channel as usize];
        let classes_before = channel.views.len();
        let Some(target) = channel.place(device, loaded.shape.mips_from(first_level)) else {
            return StreamedMips::Refused;
        };
        self.bind_group_is_stale |= channel.views.len() != classes_before;
        let target_array = &channel.textures[target.class as usize];
        for (level, bytes) in levels.iter().enumerate() {
            staging.write_texture(target_array, level as u32, target.slice, bytes);
        }
        let source = loaded.placement.chain.unwrap_or(loaded.placement.tail);
        let source_array = &channel.textures[source.class as usize];
        let first_copied_level = levels.len() as u32;
        staging.copy_texture(
            source_array,
            source.slice,
            target_array,
            target.slice,
            first_copied_level,
        );
        if let Some(chain) = loaded.placement.chain.replace(target) {
            channel.table.release(chain);
        }
        loaded.resident_level = first_level;
        let (texture_channel, layer) = (loaded.channel, target.layer());
        for material in self.loaded_materials.values() {
            if !material.textures.contains(&id) {
                continue;
            }
            let index = material.handle.index;
            let data = &mut self.materials[index as usize];
            match texture_channel {
                MaterialChannel::Emissive => data.extra[0] = layer,
                channel => data.layers[channel as usize] = layer,
            }
            let offset = u64::from(index) * std::mem::size_of::<MaterialData>() as u64;
            staging.write(&self.materials_buffer, offset, bytemuck::bytes_of(data));
        }
        StreamedMips::Moved
    }

    /// Rebuilds the bind group when loads since the last call opened Texture
    /// classes, so a scene's loads build it once: bind groups are snapshots.
    pub fn refresh_bind_group(&mut self, device: &wgpu::Device) {
        if std::mem::take(&mut self.bind_group_is_stale) {
            let started = Instant::now();
            self.bind_group = create_bind_group(
                device,
                &self.bind_group_layout,
                &self.materials_buffer,
                &self.channels,
                &self.colour_sampler,
                &self.data_sampler,
            );
            // ADR 0017's gate: the build's cost at the bound arrays' count.
            tracing::info!(
                texture_classes = self
                    .channels
                    .iter()
                    .map(|channel| channel.views.len())
                    .sum::<usize>(),
                microseconds = started.elapsed().as_secs_f64() * 1e6,
                "material bind group built"
            );
        }
    }
}

/// A trilinear, repeating sampler at `anisotropy`, which the
/// Capability floor's anisotropic filtering keeps wgpu from clamping to 1
/// (ADR 0021).
fn trilinear_sampler(device: &wgpu::Device, label: &str, anisotropy: u16) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: anisotropy,
        ..Default::default()
    })
}

/// The material bind group (`MATERIALS`): the SSBO, each Channel's
/// Texture classes, and the colour and data Channels' samplers. Bind groups
/// are snapshots, so a new Texture class needs a new one.
fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    materials_buffer: &wgpu::Buffer,
    channels: &[ChannelTextures; 5],
    colour_sampler: &wgpu::Sampler,
    data_sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let views: Vec<Vec<&wgpu::TextureView>> = channels
        .iter()
        .map(|channel| channel.views.iter().collect())
        .collect();
    let channel_resources = channels.iter().zip(&views).map(|(channel, channel_views)| {
        let resource = wgpu::BindingResource::TextureViewArray(channel_views);
        (channel.label, resource)
    });
    let samplers = [
        ("colour_sampler", colour_sampler),
        ("data_sampler", data_sampler),
    ]
    .map(|(name, sampler)| (name, wgpu::BindingResource::Sampler(sampler)));
    let resources = std::iter::once(("materials", materials_buffer.as_entire_binding()))
        .chain(channel_resources)
        .chain(samplers);
    MATERIALS.bind_group(device, layout, resources)
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::assets::pools::tests::{gpu_context, logged};
    use crate::cooked::{CookedCache, CookedMaterial};
    use crate::gpu::GpuContext;
    use crate::stats::GpuMemoryBudgets;
    use crate::stats::gpu_memory::tests::gpu_memory;

    /// A shape's first class has one slice and each later one twice the one
    /// before's, up to `SLICES_PER_CLASS`; another shape opens classes of its
    /// own between them; and a Channel holding `CLASSES_PER_CHANNEL` classes
    /// refuses a Texture only when its shape's newest class is full.
    #[test]
    fn a_shapes_classes_double_to_the_floors_slices_and_a_full_channel_refuses() {
        let shape = |width| TextureShape {
            width,
            height: 4,
            mip_level_count: 3,
            format: wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        };
        let (wall, other) = (shape(4), shape(8));
        let mut table = ClassTable::new(CLASSES_PER_CHANNEL);
        let shapes = std::iter::repeat_n(wall, 7).chain([other, wall]);
        let placed: Vec<(u32, u32)> = shapes
            .map(|shape| table.place(shape).unwrap())
            .map(|placement| (placement.class, placement.slice))
            .collect();
        assert_eq!(
            placed,
            [
                (0, 0),
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
                (3, 0),
                (4, 0)
            ]
        );
        assert_eq!(table.slices(4), 8);
        for _ in 0..2000 {
            table.place(wall).unwrap();
        }
        let wall_slices: Vec<u32> = table
            .classes
            .iter()
            .filter(|class| class.shape == wall)
            .map(|class| class.slices)
            .collect();
        assert_eq!(wall_slices[..10], [1, 2, 4, 8, 16, 32, 64, 128, 256, 256]);
        assert!(wall_slices.iter().all(|&slices| slices <= SLICES_PER_CLASS));

        let mut width = 16;
        while table.classes.len() < CLASSES_PER_CHANNEL as usize {
            table.place(shape(width)).unwrap();
            width += 4;
        }
        assert_eq!(table.place(shape(width)), None);
        assert_eq!(table.place(other), None);
        assert!(table.place(wall).is_some());
        assert!(table.place(shape(16)).is_none());
    }

    /// Two Materials sampling one Texture load it once, and a Material
    /// without that Channel samples Layer 0; each carries its normal scale.
    #[test]
    fn a_texture_two_materials_share_loads_once() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-material-pool-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        // 4 × 4 BC4: one 8-byte block at each of its three mips.
        let occlusion = CookedTexture {
            format: wgpu::TextureFormat::Bc4RUnorm,
            width: 4,
            height: 4,
            first_level: 0,
            levels: vec![vec![255; 8]; 3],
        };
        cache
            .write(AssetId(10), "key", &occlusion.to_blob(1).unwrap())
            .unwrap();
        let shared = [None, None, None, Some(AssetId(10)), None];
        for (id, textures) in [(1, shared), (2, shared), (3, [None; 5])] {
            let material = CookedMaterial {
                base_color: [1.0; 4],
                metallic: 0.0,
                roughness: 1.0,
                emissive: [0.0; 3],
                normal_scale: 0.5,
                textures,
            };
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }

        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let texture_bytes = || {
            let bytes = memory.bytes_by_kind();
            bytes[GpuMemoryKind::TexturePool as usize]
                + bytes[GpuMemoryKind::ResidentTails as usize]
        };
        let mut pool = MaterialPool::new(
            &context.device,
            &memory,
            &mut staging,
            &PoolCapacities::checked_in(),
            false,
        );
        let neutral_bytes = texture_bytes();
        let occlusion_layers: Vec<u32> = (1..=3)
            .map(|id| {
                let handle = pool
                    .load(&context.device, &mut staging, &cache, AssetId(id))
                    .unwrap();
                pool.materials[handle.index() as usize].layers[3]
            })
            .collect();
        assert_eq!(texture_bytes() - neutral_bytes, 3 * 8);
        assert_eq!(
            occlusion_layers,
            [
                SLICES_PER_CLASS,
                SLICES_PER_CLASS,
                MaterialPool::DEFAULT_LAYER
            ]
        );
        assert!(
            pool.materials
                .iter()
                .all(|material| material.metallic_roughness[2] == 0.5)
        );
        std::fs::remove_dir_all(&directory).ok();
    }

    /// A freed Material's index takes the next Material inserted, in a new
    /// generation, which the freed Handle does not find; its Asset ID loads
    /// it again.
    #[test]
    fn a_freed_materials_index_takes_the_next_material_and_refuses_the_old_handle() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-material-free-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        cache.write(AssetId(1), "key", &material.to_blob()).unwrap();
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(
            &context.device,
            &gpu_memory(&context),
            &mut staging,
            &PoolCapacities::checked_in(),
            false,
        );
        let kept = pool
            .insert(&mut staging, bytemuck::Zeroable::zeroed())
            .unwrap();
        let freed = pool
            .load(&context.device, &mut staging, &cache, AssetId(1))
            .unwrap();

        pool.free(freed);
        assert!(pool.get(freed).is_none());
        assert!(pool.get(kept).is_some());
        assert_eq!(pool.generations().frees(), 1);
        let (_, log) = logged(|| pool.free(freed));
        assert!(log.contains("Material not freed"), "not logged: {log:?}");

        let mut replacement: MaterialData = bytemuck::Zeroable::zeroed();
        replacement.base_color = [0.5; 4];
        let inserted = pool.insert(&mut staging, replacement).unwrap();
        assert_eq!(inserted.index, freed.index);
        assert!(pool.get(freed).is_none());
        assert_eq!(
            pool.get(inserted).map(|data| data.base_color),
            Some([0.5; 4])
        );
        let reloaded = pool
            .load(&context.device, &mut staging, &cache, AssetId(1))
            .unwrap();
        assert!(reloaded != freed && reloaded != inserted);
        std::fs::remove_dir_all(&directory).ok();
    }

    /// A Material freed to make room gives back both slices of its Texture,
    /// whose Resident tail starts below its first mip, and the next Texture of
    /// that shape takes them; a Texture another loaded Material samples
    /// keeps its slices.
    #[test]
    fn a_freed_materials_texture_gives_its_slices_to_the_next_of_its_shape() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-material-slices-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let format = wgpu::TextureFormat::Bc7RgbaUnormSrgb;
        let texture = CookedTexture {
            format,
            width: 512,
            height: 512,
            first_level: 0,
            levels: (0..10)
                .map(|level| vec![0; crate::cooked::level_bytes(format, 512, 512, level)])
                .collect(),
        };
        for id in [10, 11, 12] {
            cache
                .write(AssetId(id), "key", &texture.to_blob(1).unwrap())
                .unwrap();
        }
        for (id, texture_id) in [(1, 10), (2, 11), (3, 11), (4, 12)] {
            let material = CookedMaterial {
                base_color: [1.0; 4],
                metallic: 0.0,
                roughness: 1.0,
                emissive: [0.0; 3],
                normal_scale: 1.0,
                textures: [Some(AssetId(texture_id)), None, None, None, None],
            };
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MaterialPool::new(
            &context.device,
            &gpu_memory(&context),
            &mut staging,
            &PoolCapacities::checked_in(),
            false,
        );
        let mut load = |pool: &mut MaterialPool, id| {
            pool.load(&context.device, &mut staging, &cache, AssetId(id))
                .unwrap()
        };
        let placement = |pool: &MaterialPool, id| pool.loaded_textures[&AssetId(id)].placement;

        let first = load(&mut pool, 1);
        let first_placement = placement(&pool, 10);
        assert!(first_placement.chain.is_some());
        pool.remove_user(AssetId(1));
        assert!(pool.free_least_recently_used(&cache));
        assert!(pool.get(first).is_none());
        assert!(!pool.loaded_textures.contains_key(&AssetId(10)));
        assert_eq!(pool.take_frame_frees(), 2);

        let second = load(&mut pool, 2);
        assert_eq!(placement(&pool, 11), first_placement);
        assert_eq!(
            pool.get(second).map(|data| data.layers[0]),
            Some(first_placement.layer())
        );
        load(&mut pool, 3);
        pool.remove_user(AssetId(2));
        assert!(pool.free_least_recently_used(&cache));
        assert_eq!(pool.take_frame_frees(), 1);
        assert_eq!(placement(&pool, 11), first_placement);
        load(&mut pool, 4);
        assert_ne!(placement(&pool, 12), first_placement);
        assert!(
            !pool.free_least_recently_used(&cache),
            "no Material is unused"
        );
        std::fs::remove_dir_all(&directory).ok();
    }

    /// Budgets for four Materials make an SSBO of four; the next gets no
    /// Handle, so nothing draws with another Material in its place, and the
    /// refusal is logged.
    #[test]
    fn a_material_past_capacity_is_refused_and_logged() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let capacities = PoolCapacities::from_budgets(&GpuMemoryBudgets {
            texture_pool: 64 << 20,
            ..GpuMemoryBudgets::each(0)
        });
        let memory = gpu_memory(&context);
        let mut pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        assert_eq!(
            memory.bytes_by_kind()[GpuMemoryKind::SceneAndPassBuffers as usize],
            4 * std::mem::size_of::<MaterialData>() as u64
        );
        let material = MaterialData {
            base_color: [1.0; 4],
            metallic_roughness: [0.0, 1.0, 1.0, 0.0],
            emissive: [0.0, 0.0, 0.0, 1.0],
            layers: [MaterialPool::DEFAULT_LAYER; 4],
            extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
        };
        for index in 0..4 {
            let handle = pool.insert(&mut staging, material);
            assert!(handle.is_some_and(|handle| handle.index() == index));
        }
        let (handle, log) = logged(|| pool.insert(&mut staging, material));
        assert!(handle.is_none(), "a Material past capacity got a Handle");
        assert!(log.contains("WARN"), "not a warning: {log:?}");
        assert!(log.contains("Material refused"), "not logged: {log:?}");
        assert_eq!(pool.materials.len(), 4);
    }
}
