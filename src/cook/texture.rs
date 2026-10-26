//! The Texture cook (ADR 0018): a source image to a KTX2 Blob of BC blocks
//! with its full mip chain, filtered in linear light.

use anyhow::Context;
use sha2::{Digest, Sha256};

use super::CookSettings;
use crate::cooked::{
    AssetId, CookedTexture, MAX_TEXTURE_SIDE, MaterialChannel, TEXTURE_MAGIC, hex, level_bytes,
};

// intel_tex_2 0.5.0 links a prebuilt C++ library (its ASTC helpers) but not
// the C++ runtime that library needs; Linux links fail without it, and
// macOS links libc++ already. Declares no items: a link directive only.
#[cfg(target_os = "linux")]
#[link(name = "stdc++")]
unsafe extern "C" {}

/// The encoder and its profile, part of every Texture's Cook key.
const ENCODER: &str = "intel_tex_2 0.5.0 basic";

/// RGBA8 pixels, row-major, top row first.
#[derive(Clone)]
pub struct LoadedImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// One Texture to cook: an image a Material samples in one Channel.
#[derive(Clone)]
pub struct TextureSource {
    pub id: AssetId,
    pub channel: MaterialChannel,
    /// Where the image comes from, for messages.
    pub name: String,
    image: SourceImage,
    /// A metallic-roughness Texture's normal Texture, whose variance widens its
    /// roughness (Toksvig).
    normal_texture: Option<Box<TextureSource>>,
}

#[derive(Clone)]
enum SourceImage {
    /// An image file (PNG or JPEG), decoded only when its Texture is stale.
    Encoded(Vec<u8>),
    /// A procedural Texture's generated RGBA8 pixels.
    Generated(LoadedImage),
}

impl TextureSource {
    pub(super) fn encoded(
        id: AssetId,
        channel: MaterialChannel,
        name: String,
        bytes: Vec<u8>,
    ) -> Self {
        let image = SourceImage::Encoded(bytes);
        Self {
            id,
            channel,
            name,
            image,
            normal_texture: None,
        }
    }

    pub(super) fn generated(
        id: AssetId,
        channel: MaterialChannel,
        name: String,
        image: LoadedImage,
    ) -> Self {
        let image = SourceImage::Generated(image);
        Self {
            id,
            channel,
            name,
            image,
            normal_texture: None,
        }
    }

    /// This metallic-roughness Texture with its roughness widened by
    /// `normal_texture`'s variance, a dependency its Cook key then covers.
    pub(super) fn with_normal_texture(self, normal_texture: TextureSource) -> Self {
        debug_assert_eq!(self.channel, MaterialChannel::MetallicRoughness);
        Self {
            normal_texture: Some(Box::new(normal_texture)),
            ..self
        }
    }

    /// The source as RGBA8, top row first.
    pub fn decode(&self) -> anyhow::Result<LoadedImage> {
        match &self.image {
            SourceImage::Encoded(bytes) => {
                let image = image::load_from_memory(bytes)
                    .with_context(|| format!("decoding {}", self.name))?
                    .into_rgba8();
                Ok(LoadedImage {
                    width: image.width(),
                    height: image.height(),
                    pixels: image.into_raw(),
                })
            }
            SourceImage::Generated(image) => Ok(LoadedImage {
                pixels: image.pixels.clone(),
                width: image.width,
                height: image.height,
            }),
        }
    }

    /// SHA-256 over the format, the encoder, the settings, the Channel, a
    /// normal Texture's Cook key and the source bytes; a generated image's bytes
    /// are its pixels and size.
    pub(super) fn cook_key(&self, settings: &CookSettings, format_version: u32) -> String {
        let mut hasher = Sha256::new();
        hasher.update(TEXTURE_MAGIC);
        hasher.update(format_version.to_le_bytes());
        hasher.update(ENCODER);
        hasher.update(settings.texture_zstd_level.to_le_bytes());
        hasher.update([self.channel as u8]);
        // Tagged, and ahead of the source bytes, whose length varies, so that
        // no source's bytes can stand in for it.
        if let Some(normal_texture) = &self.normal_texture {
            hasher.update(b"N");
            hasher.update(normal_texture.cook_key(settings, format_version));
        }
        match &self.image {
            SourceImage::Encoded(bytes) => hasher.update(bytes),
            SourceImage::Generated(image) => {
                hasher.update(image.width.to_le_bytes());
                hasher.update(image.height.to_le_bytes());
                hasher.update(&image.pixels);
            }
        }
        hex(&hasher.finalize())
    }

    /// The Texture Blob: every mip encoded in the Channel's format, each on
    /// `encode_threads` threads.
    pub(super) fn cook(
        &self,
        settings: &CookSettings,
        encode_threads: usize,
    ) -> anyhow::Result<Vec<u8>> {
        let image = self.decode()?;
        let (width, height) = (image.width, image.height);
        anyhow::ensure!(
            width.is_multiple_of(4) && height.is_multiple_of(4),
            "{}: {width}x{height} is not a multiple of 4 on each side",
            self.name
        );
        anyhow::ensure!(
            width <= MAX_TEXTURE_SIDE && height <= MAX_TEXTURE_SIDE,
            "{}: {width}x{height} exceeds {MAX_TEXTURE_SIDE} on a side",
            self.name
        );
        let format = match self.channel {
            MaterialChannel::Albedo | MaterialChannel::Emissive => {
                wgpu::TextureFormat::Bc7RgbaUnormSrgb
            }
            MaterialChannel::Normal | MaterialChannel::MetallicRoughness => {
                wgpu::TextureFormat::Bc5RgUnorm
            }
            MaterialChannel::Occlusion => wgpu::TextureFormat::Bc4RUnorm,
        };
        let mips = width.max(height).ilog2() as usize + 1;
        let mut level = linearise(&image, self.channel);
        let mut spread = match &self.normal_texture {
            Some(normal_texture) => Some(Level::spread(&normal_texture.decode()?)),
            None => None,
        };
        let mut levels = Vec::with_capacity(mips);
        for mip in 0..mips {
            if mip > 0 {
                level = level.downsample();
            }
            if self.channel == MaterialChannel::Normal {
                level.renormalise();
            }
            // The normal Texture's level whose texels cover this level's; one
            // coarser than this level, or of another aspect ratio, widens
            // nothing here.
            let widened = spread.as_mut().and_then(|spread| {
                while spread.width > level.width || spread.height > level.height {
                    *spread = spread.downsample_spread();
                }
                (spread.width == level.width && spread.height == level.height)
                    .then(|| level.widened(spread))
            });
            let texels = match (mip, self.channel) {
                // The source's own bytes, without a round trip through
                // linear light.
                (0, MaterialChannel::Albedo | MaterialChannel::Emissive) => Texels {
                    width,
                    height,
                    bytes: image.pixels.clone(),
                },
                _ => widened.as_ref().unwrap_or(&level).to_texels(self.channel),
            };
            let blocks = encode(self.channel, format, &texels, encode_threads);
            debug_assert_eq!(blocks.len(), level_bytes(format, width, height, mip));
            levels.push(blocks);
        }
        CookedTexture {
            format,
            width,
            height,
            levels,
        }
        .to_blob(settings.texture_zstd_level)
    }
}

/// One mip level in linear light: `components` floats per texel.
struct Level {
    width: u32,
    height: u32,
    components: usize,
    values: Vec<f32>,
}

/// The source's stored channels in linear light: albedo and emissive
/// RGBA with sRGB decoded, a normal as a vector, roughness and metallic
/// (glTF's G and B), occlusion (glTF's R).
fn linearise(image: &LoadedImage, channel: MaterialChannel) -> Level {
    let srgb_to_linear: Vec<f32> = (0..=255u8)
        .map(|value| {
            let encoded = value as f32 / 255.0;
            if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    let unit = |value: u8| value as f32 / 255.0;
    let texels = image.pixels.as_chunks::<4>().0.iter();
    let (components, values): (usize, Vec<f32>) = match channel {
        MaterialChannel::Albedo | MaterialChannel::Emissive => (
            4,
            texels
                .flat_map(|&[red, green, blue, alpha]| {
                    [
                        srgb_to_linear[red as usize],
                        srgb_to_linear[green as usize],
                        srgb_to_linear[blue as usize],
                        unit(alpha),
                    ]
                })
                .collect(),
        ),
        MaterialChannel::Normal => (
            3,
            texels
                .flat_map(|&[x, y, z, _]| [x, y, z].map(|value| unit(value) * 2.0 - 1.0))
                .collect(),
        ),
        MaterialChannel::MetallicRoughness => (
            2,
            texels
                .flat_map(|&[_, roughness, metallic, _]| [unit(roughness), unit(metallic)])
                .collect(),
        ),
        MaterialChannel::Occlusion => (1, texels.map(|&[occlusion, ..]| unit(occlusion)).collect()),
    };
    Level {
        width: image.width,
        height: image.height,
        components,
        values,
    }
}

impl Level {
    /// A normal Texture's spread at its finest level: per texel the unit normal,
    /// and the variance about it of the normals the texel covers, zero.
    fn spread(normal_texture: &LoadedImage) -> Level {
        let mut normals = linearise(normal_texture, MaterialChannel::Normal);
        normals.renormalise();
        let values = normals
            .values
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[x, y, z]| [x, y, z, 0.0])
            .collect();
        Level {
            width: normals.width,
            height: normals.height,
            components: 4,
            values,
        }
    }

    /// The next mip: each texel the mean of a 2×2 box.
    fn downsample(&self) -> Level {
        self.reduce(|box_texels, values| {
            values.extend((0..self.components).map(|component| {
                box_texels.iter().map(|texel| texel[component]).sum::<f32>() / 4.0
            }));
        })
    }

    /// The next level of a spread: per texel the unnormalised mean of the
    /// normals it covers and their variance about it, 1 - |mean|², which is
    /// its box's variances plus the box means' own about it. Carried so,
    /// rather than read off the mean's length, the variance of normals that
    /// agree stays zero.
    fn downsample_spread(&self) -> Level {
        self.reduce(|box_texels, values| {
            let means = box_texels.map(glam::Vec3::from_slice);
            let mean = means.iter().sum::<glam::Vec3>() / 4.0;
            let variance = box_texels
                .iter()
                .zip(means)
                .map(|(texel, texel_mean)| texel[3] + texel_mean.distance_squared(mean))
                .sum::<f32>()
                / 4.0;
            values.extend(mean.to_array());
            values.push(variance);
        })
    }

    /// The next mip, each texel `combine`d into `values` from a 2×2 box, a
    /// side of 1 repeating its row or column.
    // An odd side drops its last row or column; a 3-tap filter there if
    // non-power-of-two Textures show it.
    fn reduce(&self, combine: impl Fn([&[f32]; 4], &mut Vec<f32>)) -> Level {
        let (width, height) = ((self.width / 2).max(1), (self.height / 2).max(1));
        let components = self.components;
        let at = |x: u32, y: u32| {
            let (x, y) = (x.min(self.width - 1), y.min(self.height - 1));
            &self.values[(y * self.width + x) as usize * components..][..components]
        };
        let mut values = Vec::with_capacity((width * height) as usize * components);
        for y in 0..height {
            for x in 0..width {
                let box_texels = [
                    at(2 * x, 2 * y),
                    at(2 * x + 1, 2 * y),
                    at(2 * x, 2 * y + 1),
                    at(2 * x + 1, 2 * y + 1),
                ];
                combine(box_texels, &mut values);
            }
        }
        Level {
            width,
            height,
            components,
            values,
        }
    }

    /// This metallic-roughness level with each texel's roughness widened by
    /// the variance in `spread`, a level of the same size.
    fn widened(&self, spread: &Level) -> Level {
        debug_assert_eq!((self.components, spread.components), (2, 4));
        debug_assert_eq!((self.width, self.height), (spread.width, spread.height));
        let values = self
            .values
            .as_chunks::<2>()
            .0
            .iter()
            .zip(spread.values.as_chunks::<4>().0)
            .flat_map(|(&[roughness, metallic], &[.., variance])| {
                [widened_roughness(roughness, variance), metallic]
            })
            .collect();
        Level {
            width: self.width,
            height: self.height,
            components: self.components,
            values,
        }
    }

    /// Scales every normal to unit length; a zero vector becomes +Z.
    fn renormalise(&mut self) {
        for normal in self.values.as_chunks_mut::<3>().0 {
            let vector = glam::Vec3::from_array(*normal)
                .try_normalize()
                .unwrap_or(glam::Vec3::Z);
            *normal = vector.to_array();
        }
    }

    /// The bytes the encoder reads, each side padded to a multiple of 4 by
    /// repeating the last row and column: RGBA8 sRGB for albedo and emissive, RG8 for a
    /// normal's x and y and for roughness and metallic, R8 for occlusion.
    fn to_texels(&self, channel: MaterialChannel) -> Texels {
        // The sRGB encoding, ≤ at the knee [ref:khronos-dfs-srgb].
        let linear_to_srgb = |linear: f32| {
            if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            }
        };
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        let (width, height) = (
            self.width.next_multiple_of(4),
            self.height.next_multiple_of(4),
        );
        let mut bytes = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let (x, y) = (x.min(self.width - 1), y.min(self.height - 1));
                let texel = &self.values[(y * self.width + x) as usize * self.components..]
                    [..self.components];
                match channel {
                    MaterialChannel::Albedo | MaterialChannel::Emissive => {
                        bytes.extend(texel[..3].iter().map(|&value| byte(linear_to_srgb(value))));
                        bytes.push(byte(texel[3]));
                    }
                    MaterialChannel::Normal => {
                        bytes.extend(texel[..2].iter().map(|&value| byte(value * 0.5 + 0.5)));
                    }
                    MaterialChannel::MetallicRoughness | MaterialChannel::Occlusion => {
                        bytes.extend(texel.iter().map(|&value| byte(value)));
                    }
                }
            }
        }
        Texels {
            width,
            height,
            bytes,
        }
    }
}

/// `roughness`, glTF's (GGX's alpha is its square
/// [ref:gltf-2.0-roughness-mapping]), widened for normals
/// whose variance about their mean is `variance` (Toksvig). It widens the
/// Texture's own roughness, as if its Material's roughness factor were 1: a
/// factor f scales the widening's share of alpha² by f⁴.
fn widened_roughness(roughness: f32, variance: f32) -> f32 {
    if variance <= 0.0 {
        return roughness;
    }
    // Toksvig, "Mipmapping Normal Maps" (NVIDIA, 2004; JGT 10(3), 2005),
    // eq. 1 [ref:toksvig-2004-variance]: the normals' angular variance from
    // their mean's length, (1 - |mean|) / |mean|, with |mean| =
    // sqrt(1 - variance) and the difference written so that it cancels
    // nothing.
    let mean_length = (1.0 - variance.min(1.0)).sqrt();
    let angular_variance = variance / ((1.0 + mean_length) * mean_length);
    // A Beckmann lobe filtered by normals of slope variance σ² adds 2σ² to
    // alpha²: Kaplanyan, Hill, Patney and Lefohn, "Filtering Distributions
    // of Normals for Shading Antialiasing" (HPG 2016), eqs. 14 and 15
    // [ref:kaplanyan-2016-ndf-filtering]; Tokuyoshi and Kaplanyan, "Improved
    // Geometric Specular Antialiasing" (I3D 2019), section 2.2, apply the same
    // sum to GGX [ref:tokuyoshi-2019-roughness-matrix]. Toksvig's angular
    // variance stands in for the slope variance, as tan θ ≈ θ for small
    // angles. Tokuyoshi and Kaplanyan keep Kaplanyan et al.'s cap of 0.18 on
    // the added term, against the error of screen-space derivatives; a
    // variance measured from the texels needs no such cap, so only alpha² is
    // capped, at 1. Equations only; no code copied.
    (roughness.powi(4) + 2.0 * angular_variance)
        .min(1.0)
        .powf(0.25)
}

/// Encoder input: tightly packed rows, each side a multiple of 4.
struct Texels {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

/// `texels` in the Channel's BC `format`, without RDO: strips of block rows
/// encode on `threads` threads, and their blocks concatenate in row order.
fn encode(
    channel: MaterialChannel,
    format: wgpu::TextureFormat,
    texels: &Texels,
    threads: usize,
) -> Vec<u8> {
    let row_bytes = texels.bytes.len() / texels.height as usize;
    let components = row_bytes as u32 / texels.width;
    let block_bytes = format.block_copy_size(None).expect("a BC format") as usize;
    let blocks_wide = (texels.width / 4) as usize;
    let block_rows = (texels.height / 4) as usize;
    let strip_block_rows = block_rows.div_ceil(threads);
    // Alpha below 255 needs the profile that keeps alpha.
    let bc7_settings = if components == 4
        && texels
            .bytes
            .as_chunks::<4>()
            .0
            .iter()
            .any(|texel| texel[3] < 255)
    {
        intel_tex_2::bc7::alpha_basic_settings()
    } else {
        intel_tex_2::bc7::opaque_basic_settings()
    };
    let mut blocks = vec![0; blocks_wide * block_rows * block_bytes];
    std::thread::scope(|scope| {
        let strips = texels.bytes.chunks(strip_block_rows * 4 * row_bytes);
        for (data, output) in
            strips.zip(blocks.chunks_mut(strip_block_rows * blocks_wide * block_bytes))
        {
            let bc7_settings = &bc7_settings;
            scope.spawn(move || {
                let (width, stride) = (texels.width, row_bytes as u32);
                let height = (data.len() / row_bytes) as u32;
                match channel {
                    MaterialChannel::Albedo | MaterialChannel::Emissive => {
                        let surface = intel_tex_2::RgbaSurface {
                            data,
                            width,
                            height,
                            stride,
                        };
                        intel_tex_2::bc7::compress_blocks_into(bc7_settings, &surface, output);
                    }
                    MaterialChannel::Normal | MaterialChannel::MetallicRoughness => {
                        let surface = intel_tex_2::RgSurface {
                            data,
                            width,
                            height,
                            stride,
                        };
                        intel_tex_2::bc5::compress_blocks_into(&surface, output);
                    }
                    MaterialChannel::Occlusion => {
                        let surface = intel_tex_2::RSurface {
                            data,
                            width,
                            height,
                            stride,
                        };
                        intel_tex_2::bc4::compress_blocks_into(&surface, output);
                    }
                }
            });
        }
    });
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cooked::TEXTURE_FORMAT_VERSION;

    #[test]
    fn the_cook_key_covers_the_format_version() {
        let image = LoadedImage {
            pixels: vec![255; 64],
            width: 4,
            height: 4,
        };
        let source =
            TextureSource::generated(AssetId(1), MaterialChannel::Albedo, "white".into(), image);
        let settings = CookSettings::default();
        assert_ne!(
            source.cook_key(&settings, TEXTURE_FORMAT_VERSION),
            source.cook_key(&settings, TEXTURE_FORMAT_VERSION + 1)
        );
    }
}
