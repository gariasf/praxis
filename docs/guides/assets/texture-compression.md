# Texture Compression Guide

This guide covers GPU-based texture compression using BC7 and BC5 formats in Praxis.

## Table of Contents

1. [Overview](#overview)
2. [Compression Formats](#compression-formats)
3. [Usage](#usage)
4. [Quality Considerations](#quality-considerations)
5. [Performance](#performance)
6. [Integration](#integration)
7. [Visual Verification](#visual-verification)

## Overview

Texture compression reduces VRAM usage by encoding pixel data into compact block formats. Praxis supports GPU-accelerated BC7 and BC5 compression using compute shaders, achieving 4:1 compression ratios with minimal quality loss.

### Benefits

- **VRAM Savings**: 75% reduction in texture memory (4:1 compression)
- **GPU Performance**: Fast compression via compute shaders (~0.5-1ms per 512×512 texture)
- **Visual Quality**: Near-lossless for procedural and natural textures
- **Runtime Compression**: Compress textures dynamically without preprocessing

### When to Use

✅ **Use compression for:**
- Procedural textures (noise, patterns)
- Natural images (photos, scanned materials)
- Normal maps (BC5 format)
- Large texture sets consuming significant VRAM
- Runtime-generated content

❌ **Avoid compression for:**
- Very small textures (<64×64)
- UI elements requiring pixel-perfect accuracy
- Textures with hard edges (text, icons)
- Textures already in compressed formats

## Compression Formats

### BC7 (RGBA Color Compression)

**Format**: `BC7_UNORM_BLOCK`  
**Compression**: 4:1 (16 bytes per 4×4 block)  
**Channels**: RGBA (full color with alpha)

```rust
use praxis_procedural::{CompressionFormat, CompressionQuality};

let format = CompressionFormat::BC7;
assert_eq!(format.block_size(), 16);
assert_eq!(format.block_dimensions(), (4, 4));
assert_eq!(format.vulkan_format(), Format::BC7_UNORM_BLOCK);
```

**Best for:**
- Albedo/base color maps
- Emissive textures
- Color gradients
- Textures with smooth color transitions
- Images with or without alpha

**Quality characteristics:**
- Mode 6 implementation (7-bit RGB endpoints, 8-bit alpha)
- 4-bit per-pixel indices (16 interpolation steps)
- Excellent for smooth gradients
- Minimal color banding
- Good alpha channel preservation

### BC5 (Two-Channel Compression)

**Format**: `BC5_UNORM_BLOCK`  
**Compression**: 4:1 (16 bytes per 4×4 block)  
**Channels**: RG (two-channel data)

```rust
use praxis_procedural::CompressionFormat;

let format = CompressionFormat::BC5;
assert_eq!(format.block_size(), 16);
assert_eq!(format.block_dimensions(), (4, 4));
assert_eq!(format.vulkan_format(), Format::BC5_UNORM_BLOCK);
```

**Best for:**
- Normal maps (RG = XY tangent space)
- Height maps
- Two-channel procedural data
- Displacement maps

**Quality characteristics:**
- Two independent BC4 compressions (one per channel)
- 8-bit endpoints per channel
- 3-bit per-pixel indices (8 interpolation steps)
- Excellent for normal map detail
- Smooth channel transitions

## Usage

### Basic Compression

```rust
use praxis_procedural::{
    TextureCompressor, CompressionFormat, CompressionQuality,
    CompressedTextureData,
};
use std::sync::Arc;

// Create compressor (requires Vulkan device and allocators)
let mut compressor = TextureCompressor::new(
    device.clone(),
    queue.clone(),
    memory_allocator.clone(),
    command_buffer_allocator.clone(),
    descriptor_set_allocator.clone(),
);

// Prepare uncompressed texture data (RGBA8 format)
let width = 512u32;
let height = 512u32;
let uncompressed_data: Vec<u8> = /* RGBA8 pixel data */;

// Compress using BC7 format with high quality
let compressed = compressor.compress(
    &uncompressed_data,
    width,
    height,
    CompressionFormat::BC7,
    CompressionQuality::High,
)?;

// Access compressed data
println!("Original size: {} bytes", width * height * 4);
println!("Compressed size: {} bytes", compressed.data.len());
println!("Compression ratio: {:.1}:1", compressed.compression_ratio());
println!("VRAM savings: {} bytes", compressed.vram_savings());
```

### Procedural Texture Compression

Integrate compression with procedural texture generation:

```rust
use praxis_procedural::{
    ProceduralTextureGenerator, TextureGraph, TextureNode, NoiseType,
    TextureGenerationParams, CompressionFormat, CompressionQuality,
    GeneratedTexture,
};

// Create texture generator
let mut generator = ProceduralTextureGenerator::new(
    device.clone(),
    queue.clone(),
    memory_allocator.clone(),
    command_buffer_allocator.clone(),
    descriptor_set_allocator.clone(),
);

// Enable compression support
generator.enable_compression();

// Create procedural texture graph
let mut graph = TextureGraph::new();
let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});
graph.set_output(noise_id);

// Generate and compress texture
let params = TextureGenerationParams {
    width: 512,
    height: 512,
    seed: 42,
    compress: true,
    compression_format: Some(CompressionFormat::BC7),
    compression_quality: Some(CompressionQuality::High),
};

let result = generator.generate_texture(&graph, params)?;

match result {
    GeneratedTexture::Compressed(compressed) => {
        println!("Generated compressed texture: {} bytes", compressed.data.len());
        println!("VRAM savings: {} KB", compressed.vram_savings() / 1024);
    }
    GeneratedTexture::Uncompressed { data, width, height } => {
        println!("Generated uncompressed texture: {} bytes", data.len());
    }
}
```

### Normal Map Compression

Use BC5 format for normal maps:

```rust
// Generate normal map (two-channel RG data)
let normal_data: Vec<u8> = /* Generate or load normal map */;

// Compress with BC5 format
let compressed_normal = compressor.compress(
    &normal_data,
    1024,
    1024,
    CompressionFormat::BC5,
    CompressionQuality::High,
)?;

// BC5 preserves RG channels for tangent space X and Y
// Blue channel (Z) can be reconstructed in shader:
// normal.z = sqrt(1.0 - normal.x*normal.x - normal.y*normal.y);
```

### Quality Settings

```rust
use praxis_procedural::CompressionQuality;

// Fast compression: Bounding box method
// - Faster compression (~0.3-0.5ms per 512×512)
// - Good quality for most textures
// - Uses min/max color bounds directly
let fast = CompressionQuality::Fast;

// High quality compression: Refined endpoints
// - Slower compression (~0.8-1.2ms per 512×512)
// - Better quality for smooth gradients
// - Uses inset technique to reduce error
let high = CompressionQuality::High;
```

## Quality Considerations

### Texture Requirements

**Dimension constraints:**
- Width and height must be multiples of 4
- Minimum size: 4×4 pixels (one block)
- No maximum size (limited by VRAM)

```rust
// Valid dimensions
let valid = [(4, 4), (512, 512), (1024, 512), (2048, 2048)];

// Invalid dimensions (not multiples of 4)
let invalid = [(100, 100), (511, 511), (1000, 1000)];
```

**Input format:**
- Must be RGBA8 format (4 bytes per pixel)
- Data layout: `[R, G, B, A, R, G, B, A, ...]` (row-major)
- Each channel: 0-255 (8-bit unsigned integer)

### Visual Quality Assessment

**Expected results at reasonable viewing distances:**

✅ **Good compression candidates:**
- Smooth gradients (noise, clouds)
- Natural textures (wood, stone, fabric)
- Low-frequency patterns
- Subtle color variations
- Normal maps with smooth surfaces

❌ **Poor compression candidates:**
- Sharp text or line art
- High-contrast edges
- Pixel art with solid colors
- UI elements with precise pixels
- Very small textures (<64×64)

**Artifacts to check for:**
- Block boundaries (4×4 grid patterns)
- Color banding in smooth gradients
- Loss of fine detail
- Color shift in saturated regions

## Performance

### Compression Speed

| Texture Size | Fast Quality | High Quality |
|--------------|--------------|--------------|
| 256×256      | ~0.2 ms      | ~0.4 ms      |
| 512×512      | ~0.5 ms      | ~1.0 ms      |
| 1024×1024    | ~1.8 ms      | ~3.5 ms      |
| 2048×2048    | ~7.0 ms      | ~14.0 ms     |

*Measured on mid-range GPU (RTX 3060)*

### Memory Savings

| Texture Size | Uncompressed | Compressed | Savings |
|--------------|--------------|------------|---------|
| 256×256      | 256 KB       | 64 KB      | 192 KB  |
| 512×512      | 1 MB         | 256 KB     | 768 KB  |
| 1024×1024    | 4 MB         | 1 MB       | 3 MB    |
| 2048×2048    | 16 MB        | 4 MB       | 12 MB   |

**Compression ratio**: Consistent 4:1 for all texture sizes

### GPU Compute Shader Details

**Workgroup organization:**
- Each workgroup processes one 4×4 block
- Workgroup size: 4×4 threads (16 threads per block)
- Thread cooperation via shared memory

**Execution model:**
1. Each thread loads one pixel into shared memory
2. Barrier synchronization
3. Thread 0 compresses the block
4. Write compressed 128-bit block to output

**Dispatch calculation:**
```glsl
dispatch_x = width / 4   // Number of blocks horizontally
dispatch_y = height / 4  // Number of blocks vertically
```

## Integration

### Vulkan Image Creation

After compression, create a Vulkan image with compressed format:

```rust
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::format::Format;

// Create compressed image
let image = Image::new(
    memory_allocator.clone(),
    ImageCreateInfo {
        image_type: ImageType::Dim2d,
        format: compressed.format.vulkan_format(), // BC7_UNORM_BLOCK
        extent: [compressed.width, compressed.height, 1],
        usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
        ..Default::default()
    },
    AllocationCreateInfo::default(),
)?;

// Upload compressed data (use staging buffer and copy)
// Note: Compressed data is uploaded directly to image memory
```

### Shader Sampling

Sample compressed textures normally in shaders:

```glsl
#version 450

layout(set = 0, binding = 0) uniform sampler2D albedoTexture;

void main() {
    // Compressed texture is automatically decompressed by hardware
    vec4 color = texture(albedoTexture, uv);
    // Use color normally
}
```

**Important**: Decompression happens transparently in hardware during texture sampling. No shader changes needed.

## Visual Verification

### Test Scene Setup

The `texture_compression_demo` example provides visual verification:

```bash
cargo run --example texture_compression_demo
```

**Scene layout:**
- Top row: 3 uncompressed procedural textures (Perlin, Simplex, Worley)
- Bottom row: 3 BC7 compressed versions of the same textures

**Controls:**
- `1` - Toggle compression on/off
- `2` - Cycle quality (Fast/High)
- `3` - Regenerate textures with new seed
- `P` - Print memory statistics

### Verification Checklist

When verifying compression quality:

✅ **Check at multiple distances:**
- Close-up view (~1-2 units from surface)
- Medium distance (~5-10 units)
- Far distance (~20+ units)

✅ **Look for artifacts:**
- Block boundaries (4×4 grid patterns)
- Color banding in gradients
- Loss of fine detail
- Compression noise

✅ **Compare directly:**
- Switch compression on/off with `1` key
- Observe any visible differences
- Check that differences are acceptable

✅ **Verify memory usage:**
- Press `P` to print statistics
- Confirm 4:1 compression ratio
- Check VRAM savings match expectations

### Expected Results

**Good compression:**
- No visible block boundaries at normal viewing distance
- Smooth color transitions preserved
- Detail remains clear and crisp
- Minimal visual difference from uncompressed

**Acceptable artifacts:**
- Very subtle smoothing in extreme close-ups
- Minor color shift in highly saturated regions
- Negligible at normal gameplay distances

## Best Practices

### When to Compress

1. **Large texture sets**: Compress when total texture memory exceeds GPU VRAM
2. **Procedural content**: Always compress runtime-generated textures
3. **Normal maps**: Use BC5 format for all normal maps
4. **Loading optimization**: Compress to reduce streaming bandwidth

### Quality vs Performance

```rust
// Choose quality based on texture type and requirements

// Fast quality for:
// - Background textures
// - Distant objects
// - High frame rate requirements
let fast_params = TextureGenerationParams {
    compress: true,
    compression_quality: Some(CompressionQuality::Fast),
    ..Default::default()
};

// High quality for:
// - Hero assets
// - Close-up viewing
// - Textures with subtle gradients
let high_params = TextureGenerationParams {
    compress: true,
    compression_quality: Some(CompressionQuality::High),
    ..Default::default()
};
```

### Caching Strategy

```rust
// Cache compressed textures to avoid redundant compression
use std::collections::HashMap;

struct TextureCache {
    compressed: HashMap<u64, CompressedTextureData>,
}

impl TextureCache {
    fn get_or_compress(
        &mut self,
        key: u64,
        data: &[u8],
        width: u32,
        height: u32,
        compressor: &mut TextureCompressor,
    ) -> Result<&CompressedTextureData> {
        if !self.compressed.contains_key(&key) {
            let compressed = compressor.compress(
                data,
                width,
                height,
                CompressionFormat::BC7,
                CompressionQuality::High,
            )?;
            self.compressed.insert(key, compressed);
        }
        Ok(self.compressed.get(&key).unwrap())
    }
}
```

## Troubleshooting

### Common Issues

**Problem**: Dimension validation error
```
Error: Texture dimensions must be multiples of 4 for block compression (got 100x100)
```
**Solution**: Ensure width and height are multiples of 4. Resize textures if needed.

---

**Problem**: Input data size mismatch
```
Error: Input data size mismatch: expected 1048576 bytes (512x512 RGBA8), got 786432
```
**Solution**: Verify input data is RGBA8 format (4 bytes per pixel). Convert RGB to RGBA if needed.

---

**Problem**: Visible block artifacts
**Solution**: 
- Use High quality mode instead of Fast
- Check that texture is suitable for compression
- Verify texture size is adequate (>128×128 recommended)

---

**Problem**: Poor normal map quality
**Solution**:
- Use BC5 format specifically for normal maps
- Ensure normal map is in tangent space with RG channels
- Increase texture resolution if detail is lost

## See Also

- [Procedural Textures Guide](procedural-textures.md)
- [Material System Guide](../rendering/materials.md)
- [VRAM Optimization](../optimization/vram.md)
- [Texture Compression Demo](../../../examples/texture_compression_demo.rs)
