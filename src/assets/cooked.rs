//! The Cooked cache as the runtime reads it (ADR 0018): Asset IDs, Blobs
//! named by the SHA-256 of their bytes, and a cook index from Asset ID to
//! Blob and Cook key, one file per Asset so concurrent cooks never lose an
//! entry.

use std::fmt;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use sha2::{Digest, Sha256};

pub use super::MaterialChannel;

/// Starts every Material Blob, followed by `MATERIAL_FORMAT_VERSION`.
pub(crate) const MATERIAL_MAGIC: [u8; 4] = *b"PXMA";
/// Bumped whenever the Material Blob layout, or how the cooker fills it,
/// changes: it stands in for the cooker version in the Cook key, and old
/// Blobs are refused.
pub(crate) const MATERIAL_FORMAT_VERSION: u32 = 1;
/// The KTX2 key of a Texture Blob's one key-value entry, whose value is
/// `TEXTURE_FORMAT_VERSION` as a little-endian u32. With its NUL it is 12
/// bytes, a multiple of the value's 4, as KTX 2.0 requires of a key whose
/// value starts with a multi-byte number [ref:ktx2-key-value-alignment].
pub(crate) const TEXTURE_VERSION_KEY: &str = "PXTXVERSION";
/// Bumped whenever the Texture Blob layout, or how the cooker fills it,
/// changes, like `MATERIAL_FORMAT_VERSION`.
pub(crate) const TEXTURE_FORMAT_VERSION: u32 = 2;
/// The largest side the cooker accepts; larger sources are refused rather
/// than resized (ADR 0018).
pub(crate) const MAX_TEXTURE_SIDE: u32 = 8192;
/// Starts every Mesh Blob, followed by `MESH_FORMAT_VERSION`.
pub(crate) const MESH_MAGIC: [u8; 4] = *b"PXME";
/// As `MATERIAL_FORMAT_VERSION`, for Mesh Blobs.
pub(crate) const MESH_FORMAT_VERSION: u32 = 2;
/// The most vertices a Meshlet holds: a packed index carries the vertex
/// within its Meshlet in six bits (ADR 0019).
pub(crate) const MESHLET_MAX_VERTICES: usize = 64;
/// The most triangles a Meshlet holds: as many as a closed mesh of
/// `MESHLET_MAX_VERTICES` vertices has (2V − 4), so that on manifold
/// geometry the vertex limit binds first.
pub(crate) const MESHLET_MAX_TRIANGLES: usize = 124;
/// The smallest QTangent w, one Snorm16 step, so the w that carries the
/// bitangent sign never rounds to zero.
pub(crate) const QTANGENT_SMALLEST_W: f32 = 1.0 / i16::MAX as f32;

/// What an Asset is. Its number, and a Texture's Channel, are part of every
/// Asset ID: never renumber.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AssetKind {
    Mesh,
    Material,
    Texture(MaterialChannel),
}

/// Where an Asset comes from.
#[derive(Copy, Clone, Debug)]
pub enum AssetSource<'a> {
    /// A manifest source file, by its GUID.
    File(u128),
    /// A procedural Reference scene, by its name.
    Scene(&'a str),
}

/// An Asset's name inside its source.
#[derive(Copy, Clone, Debug)]
pub enum AssetName<'a> {
    Name(&'a str),
    /// The glTF index, when the name is missing or not unique.
    Index(usize),
}

/// The stable name of an Asset.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetId(pub u128);

impl AssetId {
    /// The first 128 bits of SHA-256 over the source, the kind, the name
    /// and a Texture's Channel, in ADR 0018's order. Each part is tagged and each string length-prefixed, so no two
    /// inputs share an encoding.
    pub fn new(source: AssetSource, kind: AssetKind, name: AssetName) -> Self {
        let mut hasher = Sha256::new();
        match source {
            AssetSource::File(guid) => {
                hasher.update(b"G");
                hasher.update(guid.to_be_bytes());
            }
            AssetSource::Scene(scene) => {
                hasher.update(b"S");
                hash_str(&mut hasher, scene);
            }
        }
        match kind {
            AssetKind::Mesh => hasher.update([0]),
            AssetKind::Material => hasher.update([1]),
            AssetKind::Texture(_) => hasher.update([2]),
        }
        match name {
            AssetName::Name(name) => {
                hasher.update(b"N");
                hash_str(&mut hasher, name);
            }
            AssetName::Index(index) => {
                hasher.update(b"I");
                hasher.update((index as u64).to_le_bytes());
            }
        }
        if let AssetKind::Texture(channel) = kind {
            hasher.update([channel as u8]);
        }
        let digest = hasher.finalize();
        Self(u128::from_be_bytes(
            digest[..16].try_into().expect("SHA-256 is 32 bytes"),
        ))
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:032x}", self.0)
    }
}

fn hash_str(hasher: &mut Sha256, text: &str) {
    hasher.update((text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
}

/// A Material Asset: PBR factors plus one Texture Asset ID per Channel,
/// indexed by `MaterialChannel`; a missing Channel resolves to Layer 0 at
/// load.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub textures: [Option<AssetId>; 5],
}

impl CookedMaterial {
    /// Nine little-endian f32 factors, then 16 bytes per Channel, all zero
    /// for none.
    const PAYLOAD_BYTES: usize = 9 * 4 + 5 * 16;

    #[cfg(feature = "cook")]
    pub(crate) fn to_blob(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(8 + Self::PAYLOAD_BYTES);
        blob.extend(MATERIAL_MAGIC);
        blob.extend(MATERIAL_FORMAT_VERSION.to_le_bytes());
        let [red, green, blue, alpha] = self.base_color;
        let [emissive_red, emissive_green, emissive_blue] = self.emissive;
        for factor in [
            red,
            green,
            blue,
            alpha,
            self.metallic,
            self.roughness,
            emissive_red,
            emissive_green,
            emissive_blue,
        ] {
            blob.extend(factor.to_le_bytes());
        }
        for texture in self.textures {
            blob.extend(texture.map_or(0, |id| id.0).to_be_bytes());
        }
        blob
    }

    pub fn from_blob(blob: &[u8]) -> anyhow::Result<Self> {
        let payload = payload(blob, MATERIAL_MAGIC, MATERIAL_FORMAT_VERSION)?;
        anyhow::ensure!(
            payload.len() == Self::PAYLOAD_BYTES,
            "Material Blob is {} bytes, expected {}: re-cook with `praxis cook`",
            payload.len(),
            Self::PAYLOAD_BYTES
        );
        let (factor_bytes, texture_bytes) = payload.split_at(9 * 4);
        let factors: Vec<f32> = factor_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        let mut textures = [None; 5];
        for (texture, bytes) in textures.iter_mut().zip(texture_bytes.as_chunks::<16>().0) {
            let id = u128::from_be_bytes(*bytes);
            *texture = (id != 0).then_some(AssetId(id));
        }
        Ok(Self {
            base_color: factors[0..4].try_into().expect("4 factors"),
            metallic: factors[4],
            roughness: factors[5],
            emissive: factors[6..9].try_into().expect("3 factors"),
            textures,
        })
    }
}

/// A Texture Asset: one BC format per Channel (ADR 0018), with its full mip
/// chain.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTexture {
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
    /// Each mip level's blocks, mip 0 first, rows unpadded.
    pub levels: Vec<Vec<u8>>,
}

/// The formats a Texture Blob holds, with their KTX2 names.
const TEXTURE_FORMATS: [(wgpu::TextureFormat, ktx2::Format); 3] = [
    (
        wgpu::TextureFormat::Bc7RgbaUnormSrgb,
        ktx2::Format::BC7_SRGB_BLOCK,
    ),
    (
        wgpu::TextureFormat::Bc5RgUnorm,
        ktx2::Format::BC5_UNORM_BLOCK,
    ),
    (
        wgpu::TextureFormat::Bc4RUnorm,
        ktx2::Format::BC4_UNORM_BLOCK,
    ),
];

/// The bytes of mip `level` of a `width` × `height` Texture in `format`.
pub(crate) fn level_bytes(
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    level: usize,
) -> usize {
    let (block_width, block_height) = format.block_dimensions();
    let blocks_wide = (width >> level).max(1).div_ceil(block_width);
    let blocks_high = (height >> level).max(1).div_ceil(block_height);
    let block_bytes = format.block_copy_size(None).expect("a colour format");
    (blocks_wide * blocks_high * block_bytes) as usize
}

impl CookedTexture {
    /// KTX2: the header, level index and data format descriptor through
    /// `ktx2`'s serialisers, then the key-value data, then each level
    /// zstd-compressed at `zstd_level`, smallest mip first, so the smallest
    /// mips are one contiguous byte range (ADR 0017). KTX2 orders level data
    /// this way [ref:ktx2-level-order], compresses each level on its own
    /// [ref:ktx2-supercompression-per-level] and needs no level alignment
    /// under supercompression [ref:ktx2-level-alignment]. The one key-value
    /// entry's key is sized so that its u32 value lands aligned
    /// [ref:ktx2-key-value-alignment].
    #[cfg(feature = "cook")]
    pub fn to_blob(&self, zstd_level: i32) -> anyhow::Result<Vec<u8>> {
        let (_, format) = TEXTURE_FORMATS
            .into_iter()
            .find(|(format, _)| *format == self.format)
            .context("not a Texture Blob format")?;
        let (descriptor, type_size) =
            ktx2::dfd::Basic::from_format(format).expect("BC formats have a descriptor");
        let descriptor = ktx2::dfd::Block::Basic(descriptor).to_vec();
        // One entry: its length, the key, a NUL, the value, padded to 4.
        let mut key_value_entry = Vec::new();
        key_value_entry.extend((TEXTURE_VERSION_KEY.len() as u32 + 1 + 4).to_le_bytes());
        key_value_entry.extend(TEXTURE_VERSION_KEY.as_bytes());
        key_value_entry.push(0);
        key_value_entry.extend(TEXTURE_FORMAT_VERSION.to_le_bytes());
        key_value_entry.resize(key_value_entry.len().next_multiple_of(4), 0);

        let index_bytes = self.levels.len() * ktx2::LevelIndex::LENGTH;
        let descriptor_offset = ktx2::Header::LENGTH + index_bytes;
        let descriptor_bytes = 4 + descriptor.len();
        let key_value_offset = descriptor_offset + descriptor_bytes;
        let mut level_index = vec![[0; ktx2::LevelIndex::LENGTH]; self.levels.len()];
        let mut data = Vec::new();
        let data_offset = key_value_offset + key_value_entry.len();
        for (level, texels) in self.levels.iter().enumerate().rev() {
            let compressed = zstd::bulk::compress(texels, zstd_level)?;
            level_index[level] = ktx2::LevelIndex {
                byte_offset: (data_offset + data.len()) as u64,
                byte_length: compressed.len() as u64,
                uncompressed_byte_length: texels.len() as u64,
            }
            .as_bytes();
            data.extend(compressed);
        }
        let header = ktx2::Header {
            format: Some(format),
            type_size,
            pixel_width: self.width,
            pixel_height: self.height,
            pixel_depth: 0,
            layer_count: 0,
            face_count: 1,
            level_count: self.levels.len() as u32,
            supercompression_scheme: Some(ktx2::SupercompressionScheme::Zstandard),
            index: ktx2::Index {
                dfd_byte_offset: descriptor_offset as u32,
                dfd_byte_length: descriptor_bytes as u32,
                kvd_byte_offset: key_value_offset as u32,
                kvd_byte_length: key_value_entry.len() as u32,
                sgd_byte_offset: 0,
                sgd_byte_length: 0,
            },
        };
        let mut blob = header.as_bytes().to_vec();
        blob.extend(level_index.concat());
        blob.extend((descriptor_bytes as u32).to_le_bytes());
        blob.extend(descriptor);
        blob.extend(key_value_entry);
        blob.extend(data);
        Ok(blob)
    }

    /// Parses a Texture Blob back into its levels, refusing one from another
    /// format version, in another format or without its full mip chain.
    pub fn from_blob(blob: &[u8]) -> anyhow::Result<Self> {
        let reader = ktx2::Reader::new(blob).map_err(|error| {
            anyhow::anyhow!("Texture Blob is not KTX2 ({error}): re-cook with `praxis cook`")
        })?;
        let version = reader
            .key_value_data()
            .find(|(key, _)| *key == TEXTURE_VERSION_KEY)
            .and_then(|(_, value)| value.try_into().ok())
            .map(u32::from_le_bytes);
        anyhow::ensure!(
            version == Some(TEXTURE_FORMAT_VERSION),
            "Blob is not a Texture Blob of version {TEXTURE_FORMAT_VERSION} ({TEXTURE_VERSION_KEY}): re-cook with `praxis cook`"
        );
        let header = reader.header();
        anyhow::ensure!(
            header.supercompression_scheme == Some(ktx2::SupercompressionScheme::Zstandard),
            "Texture Blob levels are not zstd-compressed"
        );
        let (format, _) = TEXTURE_FORMATS
            .into_iter()
            .find(|(_, format)| Some(*format) == header.format)
            .with_context(|| {
                format!(
                    "Texture Blob format {:?} is not a Texture format",
                    header.format
                )
            })?;
        // As the cooker writes them, so that every Texture a Blob holds is
        // one wgpu creates.
        let (width, height) = (header.pixel_width, header.pixel_height);
        anyhow::ensure!(
            height > 0
                && width.is_multiple_of(4)
                && height.is_multiple_of(4)
                && width.max(height) <= MAX_TEXTURE_SIDE,
            "Texture Blob is {width}x{height}: re-cook with `praxis cook`"
        );
        let mips = width.max(height).ilog2() + 1;
        anyhow::ensure!(
            header.level_count == mips,
            "Texture Blob has {} mips, expected {mips}",
            header.level_count
        );
        let levels = reader
            .levels()
            .enumerate()
            .map(|(level, data)| {
                let expected = level_bytes(format, header.pixel_width, header.pixel_height, level);
                let texels = zstd::bulk::decompress(data.data, expected)
                    .with_context(|| format!("decompressing mip {level}"))?;
                anyhow::ensure!(
                    texels.len() == expected,
                    "mip {level} is {} bytes, expected {expected}",
                    texels.len()
                );
                Ok(texels)
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            format,
            width: header.pixel_width,
            height: header.pixel_height,
            levels,
        })
    }
}

/// A cooked vertex: 28 bytes (ADR 0018).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CookedVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    /// The rotation from tangent space (tangent, bitangent, normal) to mesh
    /// space as a unit quaternion `xyzw` in Snorm16. Its w is never zero, and
    /// its sign is the bitangent sign.
    pub qtangent: [i16; 4],
}

const _: () = assert!(size_of::<CookedVertex>() == 28);

/// One Material's run of a Mesh's indices.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedPrimitive {
    pub first_index: u32,
    pub index_count: u32,
    pub material: Option<AssetId>,
    /// Its triangles in index order, each in exactly one Meshlet.
    pub meshlets: Vec<CookedMeshlet>,
}

/// The next run of its Primitive's triangles, on at most
/// `MESHLET_MAX_VERTICES` vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMeshlet {
    /// The Mesh vertices its triangles use: its vertex remap.
    pub vertices: Vec<u32>,
    /// Its triangles as indices into `vertices`.
    pub triangles: Vec<[u8; 3]>,
}

impl CookedMeshlet {
    /// The Meshlets of `triangles`, one per run of `triangle_counts`. Each
    /// numbers its vertices in order of first use, each triangle's corners
    /// taken lowest first, so that the numbering survives the index codec
    /// rotating a triangle.
    pub(crate) fn split(
        triangles: &[[u32; 3]],
        triangle_counts: impl IntoIterator<Item = usize>,
    ) -> anyhow::Result<Vec<Self>> {
        // Each vertex's index in the current Meshlet, or u8::MAX, from the
        // lowest vertex the triangles use to the highest.
        let mesh_vertices = triangles.as_flattened();
        let lowest = mesh_vertices.iter().copied().min().unwrap_or(0);
        let highest = mesh_vertices.iter().copied().max().unwrap_or(0);
        let mut index_in_meshlet = vec![u8::MAX; (highest - lowest) as usize + 1];
        let offset = |vertex: u32| (vertex - lowest) as usize;
        let mut rest = triangles;
        let mut meshlets = Vec::new();
        for triangle_count in triangle_counts {
            anyhow::ensure!(
                (1..=MESHLET_MAX_TRIANGLES).contains(&triangle_count),
                "Mesh Blob has a Meshlet of {triangle_count} triangles"
            );
            let (run, after) = rest
                .split_at_checked(triangle_count)
                .context("Mesh Blob has a Meshlet past its Primitive's triangles")?;
            rest = after;
            let mut vertices = Vec::new();
            for triangle in run {
                let mut corners = *triangle;
                corners.sort_unstable();
                for vertex in corners {
                    if index_in_meshlet[offset(vertex)] == u8::MAX {
                        anyhow::ensure!(
                            vertices.len() < MESHLET_MAX_VERTICES,
                            "Mesh Blob has a Meshlet of more than {MESHLET_MAX_VERTICES} vertices"
                        );
                        index_in_meshlet[offset(vertex)] = vertices.len() as u8;
                        vertices.push(vertex);
                    }
                }
            }
            let local_triangles = run
                .iter()
                .map(|triangle| triangle.map(|vertex| index_in_meshlet[offset(vertex)]))
                .collect();
            for &vertex in &vertices {
                index_in_meshlet[offset(vertex)] = u8::MAX;
            }
            meshlets.push(Self {
                vertices,
                triangles: local_triangles,
            });
        }
        anyhow::ensure!(
            rest.is_empty(),
            "Mesh Blob has a Primitive's triangle in no Meshlet"
        );
        Ok(meshlets)
    }
}

/// A Mesh Asset in mesh-local space: welded vertices, u32 indices into them
/// and the Primitives that split the indices by Material.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMesh {
    pub vertices: Vec<CookedVertex>,
    pub indices: Vec<u32>,
    pub primitives: Vec<CookedPrimitive>,
    /// The box that holds every vertex: its lowest corner, then its highest.
    pub bounds: [[f32; 3]; 2],
}

impl CookedMesh {
    /// The zstd level the cooker compresses at; part of the Cook key.
    #[cfg(feature = "cook")]
    pub(crate) const ZSTD_LEVEL: i32 = 19;

    /// After the magic and version, one zstd frame holding the vertex,
    /// index and Primitive counts as little-endian u32s, the bounds as six
    /// little-endian f32s, each Primitive's first index, index count,
    /// Material Asset ID (zero for none) and Meshlet count, then a byte per
    /// Meshlet, its triangle count, the byte length of the encoded vertices,
    /// then meshopt's encoded vertices and indices. Each Meshlet's vertices
    /// and triangles are its run of the indices, rebuilt on reading.
    #[cfg(feature = "cook")]
    pub(crate) fn to_blob(&self) -> anyhow::Result<Vec<u8>> {
        let vertices = meshopt::encode_vertex_buffer(&self.vertices)?;
        let indices = meshopt::encode_index_buffer(&self.indices, self.vertices.len())?;
        let mut payload = Vec::new();
        for count in [
            self.vertices.len(),
            self.indices.len(),
            self.primitives.len(),
        ] {
            payload.extend((count as u32).to_le_bytes());
        }
        for coordinate in self.bounds.as_flattened() {
            payload.extend(coordinate.to_le_bytes());
        }
        for primitive in &self.primitives {
            payload.extend(primitive.first_index.to_le_bytes());
            payload.extend(primitive.index_count.to_le_bytes());
            payload.extend(primitive.material.map_or(0, |id| id.0).to_be_bytes());
            payload.extend((primitive.meshlets.len() as u32).to_le_bytes());
        }
        for meshlet in self
            .primitives
            .iter()
            .flat_map(|primitive| &primitive.meshlets)
        {
            payload.push(u8::try_from(meshlet.triangles.len())?);
        }
        payload.extend((vertices.len() as u32).to_le_bytes());
        payload.extend(vertices);
        payload.extend(indices);
        let mut blob = Vec::from(MESH_MAGIC);
        blob.extend(MESH_FORMAT_VERSION.to_le_bytes());
        blob.extend(zstd::bulk::compress(&payload, Self::ZSTD_LEVEL)?);
        Ok(blob)
    }

    pub fn from_blob(blob: &[u8]) -> anyhow::Result<Self> {
        let payload = zstd::decode_all(payload(blob, MESH_MAGIC, MESH_FORMAT_VERSION)?)?;
        let mut rest = payload.as_slice();
        let rest = &mut rest;
        let vertex_count = take_u32(rest)? as usize;
        let index_count = take_u32(rest)? as usize;
        // meshopt's index decoder writes whole triangles and checks neither.
        anyhow::ensure!(
            index_count.is_multiple_of(3),
            "Mesh Blob's index count is not whole triangles"
        );
        let primitive_count = take_u32(rest)?;
        let mut bounds = [[0.0; 3]; 2];
        for coordinate in bounds.as_flattened_mut() {
            *coordinate = f32::from_bits(take_u32(rest)?);
        }
        let mut primitives = Vec::new();
        let mut meshlet_counts = Vec::new();
        for _ in 0..primitive_count {
            let first_index = take_u32(rest)?;
            let primitive_index_count = take_u32(rest)?;
            let material = u128::from_be_bytes(take(rest, 16)?.try_into().expect("16 bytes"));
            anyhow::ensure!(
                first_index as usize + primitive_index_count as usize <= index_count,
                "Mesh Blob has a Primitive past its indices"
            );
            meshlet_counts.push(take_u32(rest)? as usize);
            primitives.push(CookedPrimitive {
                first_index,
                index_count: primitive_index_count,
                material: (material != 0).then_some(AssetId(material)),
                meshlets: Vec::new(),
            });
        }
        // Each Primitive's Meshlets' triangle counts.
        let meshlet_triangle_counts: Vec<&[u8]> = meshlet_counts
            .into_iter()
            .map(|meshlet_count| take(rest, meshlet_count))
            .collect::<anyhow::Result<_>>()?;
        let vertex_bytes = take_u32(rest)? as usize;
        let vertices = meshopt::decode_vertex_buffer(take(rest, vertex_bytes)?, vertex_count)?;
        // The index codec spends at least a byte per triangle, so a count
        // past that is corrupt and must not size an allocation.
        anyhow::ensure!(
            index_count / 3 <= rest.len(),
            "Mesh Blob has more indices than its encoded bytes hold"
        );
        let indices: Vec<u32> = meshopt::decode_index_buffer(rest, index_count)?;
        // meshopt's decoder may turn corrupt input into any index.
        anyhow::ensure!(
            indices.iter().all(|&index| (index as usize) < vertex_count),
            "Mesh Blob has an index past its vertices"
        );
        for (primitive, triangle_counts) in primitives.iter_mut().zip(meshlet_triangle_counts) {
            let first_index = primitive.first_index as usize;
            let (triangles, partial) =
                indices[first_index..][..primitive.index_count as usize].as_chunks::<3>();
            anyhow::ensure!(
                partial.is_empty(),
                "Mesh Blob has a Primitive of partial triangles"
            );
            primitive.meshlets = CookedMeshlet::split(
                triangles,
                triangle_counts.iter().map(|&count| count as usize),
            )?;
        }
        Ok(Self {
            vertices,
            indices,
            primitives,
            bounds,
        })
    }
}

/// The first `length` bytes of `rest`, which then starts after them.
fn take<'a>(rest: &mut &'a [u8], length: usize) -> anyhow::Result<&'a [u8]> {
    let (taken, after) = rest
        .split_at_checked(length)
        .context("Mesh Blob is truncated")?;
    *rest = after;
    Ok(taken)
}

fn take_u32(rest: &mut &[u8]) -> anyhow::Result<u32> {
    Ok(u32::from_le_bytes(
        take(rest, 4)?.try_into().expect("4 bytes"),
    ))
}

/// `blob` after its magic and format version, or a refusal asking for a
/// re-cook when either does not match: there is no migration code.
fn payload(blob: &[u8], magic: [u8; 4], version: u32) -> anyhow::Result<&[u8]> {
    let found_magic = blob.get(..4);
    let found_version = blob
        .get(4..8)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("4 bytes")));
    anyhow::ensure!(
        found_magic == Some(&magic[..]) && found_version == Some(version),
        "Blob is not a {} version {version} Blob: re-cook with `praxis cook`",
        String::from_utf8_lossy(&magic)
    );
    Ok(&blob[8..])
}

/// The Cooked cache a launch loads from, relative to the working directory.
pub const COOKED_CACHE_DIRECTORY: &str = "cooked";

/// The Cooked cache in a directory: `blobs/<SHA-256 of the Blob>` and
/// `index/<Asset ID>`, which holds the Blob's hash and the Cook key.
pub struct CookedCache {
    root: PathBuf,
}

/// One cook index entry, as lowercase hex.
pub(crate) struct IndexEntry {
    pub blob: String,
    // Read by the cooker only.
    #[cfg_attr(not(feature = "cook"), allow(dead_code))]
    pub cook_key: String,
}

impl CookedCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn blob_path(&self, blob: &str) -> PathBuf {
        self.root.join("blobs").join(blob)
    }

    fn index_path(&self, id: AssetId) -> PathBuf {
        self.root.join("index").join(id.to_string())
    }

    /// The index entry for `id` whose Blob is present, if any.
    pub(crate) fn entry(&self, id: AssetId) -> Option<IndexEntry> {
        let text = fs::read_to_string(self.index_path(id)).ok()?;
        let (blob, cook_key) = text.trim_end().split_once(' ')?;
        self.blob_path(blob).exists().then(|| IndexEntry {
            blob: blob.to_string(),
            cook_key: cook_key.to_string(),
        })
    }

    /// The Blob the index names for `id`.
    pub fn read(&self, id: AssetId) -> anyhow::Result<Vec<u8>> {
        let entry = self.entry(id).with_context(|| {
            format!(
                "Asset {id} is not in the Cooked cache at {}: run `praxis cook`",
                self.root.display()
            )
        })?;
        let path = self.blob_path(&entry.blob);
        fs::read(&path).with_context(|| format!("reading Blob {}", path.display()))
    }

    pub fn material(&self, id: AssetId) -> anyhow::Result<CookedMaterial> {
        CookedMaterial::from_blob(&self.read(id)?).with_context(|| format!("reading Material {id}"))
    }

    pub fn texture(&self, id: AssetId) -> anyhow::Result<CookedTexture> {
        CookedTexture::from_blob(&self.read(id)?).with_context(|| format!("reading Texture {id}"))
    }

    pub fn mesh(&self, id: AssetId) -> anyhow::Result<CookedMesh> {
        CookedMesh::from_blob(&self.read(id)?).with_context(|| format!("reading Mesh {id}"))
    }

    /// Stores `blob` under its hash and points `id` at it with `cook_key`.
    /// Each file is written whole and renamed into place, so a reader or a
    /// concurrent cook sees the old file or the new one, never a torn one.
    #[cfg(feature = "cook")]
    pub(crate) fn write(&self, id: AssetId, cook_key: &str, blob: &[u8]) -> anyhow::Result<()> {
        let hash = hex(&Sha256::digest(blob));
        let blob_path = self.blob_path(&hash);
        if !blob_path.exists() {
            write_atomically(&blob_path, blob)?;
        }
        write_atomically(
            &self.index_path(id),
            format!("{hash} {cook_key}\n").as_bytes(),
        )
    }
}

#[cfg(feature = "cook")]
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(feature = "cook")]
fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    // Unique per process and per write, so concurrent writers never share
    // a temporary file.
    static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
    let directory = path.parent().expect("cache paths have a parent");
    fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
    let temporary = directory.join(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    // Synced before the rename, so a crash cannot leave an empty file under
    // a name the next cook trusts.
    let mut file = fs::File::create(&temporary)
        .with_context(|| format!("creating {}", temporary.display()))?;
    std::io::Write::write_all(&mut file, bytes)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("writing {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("renaming into {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An oracle independent of the code above: SHA-256 of the encoding
    /// `G`, 16 GUID bytes, kind 1, `N`, length 8 as u64 LE, `material`,
    /// computed with `shasum -a 256` and truncated to 128 bits.
    #[test]
    fn asset_id_is_truncated_sha256_of_its_encoding() {
        let id = AssetId::new(
            AssetSource::File(0x000102030405060708090a0b0c0d0e0f),
            AssetKind::Material,
            AssetName::Name("material"),
        );
        assert_eq!(id.to_string(), "397412b2d75d55daad25a42ab0cbbb05");
    }

    #[cfg(feature = "cook")]
    #[test]
    fn material_blob_round_trips_and_refuses_another_version() {
        let material = CookedMaterial {
            base_color: [0.1, 0.2, 0.3, 1.0],
            metallic: 0.5,
            roughness: 0.25,
            emissive: [1.0, 0.0, 2.0],
            textures: [Some(AssetId(7)), None, None, Some(AssetId(u128::MAX)), None],
        };
        let blob = material.to_blob();
        assert_eq!(CookedMaterial::from_blob(&blob).unwrap(), material);

        for byte in [0, 4] {
            let mut mismatched = blob.clone();
            mismatched[byte] += 1;
            let error = CookedMaterial::from_blob(&mismatched)
                .unwrap_err()
                .to_string();
            assert!(error.contains("re-cook with `praxis cook`"), "{error}");
        }
    }

    #[cfg(feature = "cook")]
    #[test]
    fn texture_blob_round_trips_smallest_mip_first_and_refuses_another_version() {
        // 8 × 4 BC4: two blocks at mip 0, one at mips 1 to 3.
        let texture = CookedTexture {
            format: wgpu::TextureFormat::Bc4RUnorm,
            width: 8,
            height: 4,
            levels: (0..4u8)
                .map(|mip| vec![mip; if mip == 0 { 16 } else { 8 }])
                .collect(),
        };
        let blob = texture.to_blob(3).unwrap();
        assert_eq!(CookedTexture::from_blob(&blob).unwrap(), texture);

        let reader = ktx2::Reader::new(&blob[..]).unwrap();
        let offsets: Vec<u64> = (0..4)
            .map(|level| {
                let entry = &blob[ktx2::Header::LENGTH + level * ktx2::LevelIndex::LENGTH..];
                u64::from_le_bytes(entry[..8].try_into().unwrap())
            })
            .collect();
        assert!(
            offsets.is_sorted_by(|larger, smaller| larger > smaller),
            "{offsets:?}"
        );
        assert_eq!(reader.levels().len(), 4);

        // KTX 2.0 aligns a value that starts with a multi-byte number by
        // requiring the key and its NUL to be a multiple of the number's
        // size [ref:ktx2-key-value-alignment].
        let key = [TEXTURE_VERSION_KEY.as_bytes(), &[0]].concat();
        assert!(key.len().is_multiple_of(4), "{TEXTURE_VERSION_KEY:?}");
        let version_at = blob
            .windows(key.len())
            .position(|window| window == key)
            .unwrap()
            + key.len();
        assert!(version_at.is_multiple_of(4), "{version_at}");

        // M3's first Texture Blobs, whose key was `PXTX`.
        let mut old_version = blob.clone();
        old_version[version_at..version_at + 4].copy_from_slice(&1u32.to_le_bytes());
        let error = CookedTexture::from_blob(&old_version)
            .unwrap_err()
            .to_string();
        assert!(error.contains("re-cook with `praxis cook`"), "{error}");

        // wgpu creates no BC Texture 6 texels wide.
        let unaligned = CookedTexture {
            width: 6,
            ..texture
        }
        .to_blob(3)
        .unwrap();
        let error = CookedTexture::from_blob(&unaligned)
            .unwrap_err()
            .to_string();
        assert!(error.contains("6x4: re-cook"), "{error}");
    }

    /// M2's Mesh Blobs, which hold no Meshlets.
    #[cfg(feature = "cook")]
    #[test]
    fn mesh_blob_of_format_version_1_is_refused_with_a_re_cook_message() {
        let mesh = CookedMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            primitives: Vec::new(),
            bounds: [[0.0; 3]; 2],
        };
        let mut blob = mesh.to_blob().unwrap();
        blob[4..8].copy_from_slice(&1u32.to_le_bytes());
        let error = CookedMesh::from_blob(&blob).unwrap_err().to_string();
        assert!(error.contains("re-cook with `praxis cook`"), "{error}");
    }

    #[cfg(feature = "cook")]
    #[test]
    fn mesh_blob_with_a_partial_triangle_is_refused() {
        let mesh = CookedMesh {
            vertices: vec![CookedVertex::default(); 3],
            indices: vec![0, 1, 2],
            primitives: Vec::new(),
            bounds: [[0.0; 3]; 2],
        };
        let blob = mesh.to_blob().unwrap();
        let mut payload = zstd::decode_all(&blob[8..]).unwrap();
        // The index count, after the vertex count.
        payload[4..8].copy_from_slice(&4u32.to_le_bytes());
        let mut corrupt = blob[..8].to_vec();
        corrupt.extend(zstd::bulk::compress(&payload, 1).unwrap());
        let error = CookedMesh::from_blob(&corrupt).unwrap_err().to_string();
        assert!(error.contains("not whole triangles"), "{error}");
    }
}
