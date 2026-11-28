use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// Probes along X, Y and Z of a Cascade.
pub(crate) const PROBE_COUNTS: [u32; 3] = [32, 16, 32];

pub(crate) const PROBE_SPACING_METRES: f32 = 1.0;

/// Texels along a side of a Probe's octahedral map of irradiance, and of its
/// depth moments [ref:majercik-2019-ddgi-probe-encoding].
const IRRADIANCE_SIDE_TEXELS: u32 = 8;
const DEPTH_MOMENTS_SIDE_TEXELS: u32 = 16;

/// Blendable; the Capability floor gives neither read-write storage
/// [ref:wgpu-types-30-guaranteed-format-features].
const IRRADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_MOMENTS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

const STATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;

/// One Cascade's Probes (ADR 0020). Each texture is one layer: a Probe's map
/// lies at its Z down the texture and its X across it, the levels along Y
/// side by side.
pub(crate) struct Cascade {
    irradiance: Counted<wgpu::Texture>,
    #[expect(dead_code, reason = "created and counted; no pass binds it")]
    depth_moments: Counted<wgpu::Texture>,
    #[expect(dead_code, reason = "created and counted; no pass binds it")]
    state: Counted<wgpu::Texture>,
}

impl Cascade {
    pub(crate) fn new(device: &wgpu::Device, memory: &GpuMemory) -> Cascade {
        let [across, levels, deep] = PROBE_COUNTS;
        let texture = |label, side_texels, format, usage| {
            memory.create_texture(
                device,
                GpuMemoryKind::Probes,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: across * levels * side_texels,
                        height: deep * side_texels,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                },
            )
        };
        let binding = wgpu::TextureUsages::TEXTURE_BINDING;
        Cascade {
            irradiance: texture(
                "Probe Irradiance",
                IRRADIANCE_SIDE_TEXELS,
                IRRADIANCE_FORMAT,
                binding | wgpu::TextureUsages::COPY_SRC,
            ),
            depth_moments: texture(
                "Probe Depth Moments",
                DEPTH_MOMENTS_SIDE_TEXELS,
                DEPTH_MOMENTS_FORMAT,
                binding,
            ),
            state: texture("Probe State", 1, STATE_FORMAT, binding),
        }
    }

    pub(crate) fn irradiance(&self) -> &wgpu::Texture {
        &self.irradiance
    }
}

/// The irradiance each Probe holds, as stored: linear RGB at each texel of
/// its map, row by row, the Probes ordered by X within Z within Y. `texels`
/// is a Cascade's irradiance, rows top first without padding.
pub(crate) fn irradiance_by_probe(texels: &[u8]) -> Vec<Vec<[f32; 3]>> {
    let [across, levels, deep] = PROBE_COUNTS.map(|count| count as usize);
    let side = IRRADIANCE_SIDE_TEXELS as usize;
    let (texels, _) = texels.as_chunks::<8>();
    assert_eq!(texels.len(), across * levels * deep * side * side);
    let probe = |probe_index: usize| {
        let (level, row, column) = (
            probe_index / (across * deep),
            probe_index / across % deep,
            probe_index % across,
        );
        let texel = |texel_index: usize| {
            let (texel_row, texel_column) = (texel_index / side, texel_index % side);
            let texture_row = row * side + texel_row;
            let texture_column = (level * across + column) * side + texel_column;
            let rgba = texels[texture_row * across * levels * side + texture_column];
            [0, 2, 4].map(|byte_offset| {
                fp16_value(u16::from_le_bytes([
                    rgba[byte_offset],
                    rgba[byte_offset + 1],
                ]))
            })
        };
        (0..side * side).map(texel).collect()
    };
    (0..across * levels * deep).map(probe).collect()
}

/// The value of an IEEE 754 binary16's bits.
fn fp16_value(bits: u16) -> f32 {
    let (exponent, fraction) = (i32::from(bits >> 10 & 0x1f), bits & 0x3ff);
    let magnitude = match exponent {
        0 => f32::from(fraction) * 2f32.powi(-24),
        31 if fraction == 0 => f32::INFINITY,
        31 => f32::NAN,
        _ => (1.0 + f32::from(fraction) / 1024.0) * 2f32.powi(exponent - 15),
    };
    if bits >> 15 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fp16_bits_read_as_their_values() {
        assert_eq!(fp16_value(0x0000), 0.0);
        assert_eq!(fp16_value(0x0001), 5.960_464_5e-8);
        assert_eq!(fp16_value(0x3c00), 1.0);
        assert_eq!(fp16_value(0xc000), -2.0);
        assert_eq!(fp16_value(0x7bff), 65504.0);
        assert_eq!(fp16_value(0xfc00), f32::NEG_INFINITY);
        assert!(fp16_value(0x7e00).is_nan());
    }

    /// One texel lit: in the second level's Probe 3 along X and 1 along Z,
    /// at the third row and sixth column of its map.
    #[test]
    fn a_texel_of_the_irradiance_reads_back_in_its_probe() {
        let (width, height) = (32 * 16 * 8, 32 * 8);
        let mut texels = vec![0; width * height * 8];
        let lit_texel = (8 + 2) * width + (32 + 3) * 8 + 5;
        texels[lit_texel * 8..][..6].copy_from_slice(&[0x00, 0x3c, 0x00, 0x40, 0x00, 0x42]);

        let irradiance = irradiance_by_probe(&texels);

        assert_eq!(irradiance.len(), 32 * 16 * 32);
        assert!(irradiance.iter().all(|probe| probe.len() == 8 * 8));
        assert_eq!(irradiance[32 * 32 + 32 + 3][2 * 8 + 5], [1.0, 2.0, 3.0]);
        let lit_texels = irradiance
            .iter()
            .flatten()
            .filter(|&&texel| texel != [0.0; 3]);
        assert_eq!(lit_texels.count(), 1);
    }
}
