# Hardware Tier Detection System

The hardware tier detection system automatically analyzes GPU capabilities via Vulkan and recommends optimized quality presets for different hardware configurations.

## Overview

The system queries GPU properties including:
- **VRAM**: Total device-local memory available
- **Vendor**: AMD, NVIDIA, Intel, Apple, ARM, Qualcomm
- **Device Type**: Discrete, integrated, virtual
- **Compute Capability**: Estimated shader core count
- **Feature Support**: Descriptor indexing, timeline semaphores, etc.

Based on these properties, GPUs are classified into four tiers, each with corresponding quality presets.

## Hardware Tiers

### Integrated
**Target Hardware**: Intel UHD, AMD Vega (integrated), basic Apple Silicon
- Low-power integrated GPUs
- Limited VRAM (shared system memory)
- Minimal compute capability
- **Recommended Preset**: Low

### Mobile
**Target Hardware**: NVIDIA GTX 1650 Mobile, AMD RX 6600M, mid-range laptop GPUs
- Mid-range mobile/laptop GPUs
- Moderate VRAM (2-4 GB)
- Moderate compute capability
- **Recommended Preset**: Medium

### Mid-Range
**Target Hardware**: NVIDIA GTX 1660/RTX 3060, AMD RX 5700/6700 XT
- Desktop GPUs from 2-4 years ago
- Good VRAM (6-10 GB)
- Good compute capability
- **Recommended Preset**: High

### High-End
**Target Hardware**: NVIDIA RTX 3080/4080, AMD RX 6900 XT/7900 XT
- Modern high-performance GPUs
- Excellent VRAM (12+ GB)
- High compute capability
- **Recommended Preset**: Ultra

## Quality Presets

Each preset provides optimized settings for rendering features:

### Low (Integrated GPUs)
```rust
shadow_resolution: 512
shadow_cascades: 2
enable_soft_shadows: false
lod_bias: -0.5
msaa_samples: 1
enable_post_processing: false
enable_bloom: false
enable_ssao: false
enable_ssr: false
enable_taa: false
max_particles: 1000
view_distance_multiplier: 0.5
target_fps: 30.0
```

**Focus**: Maximum performance, minimal visual quality
- Disabled expensive effects
- Aggressive LOD bias
- Reduced view distance
- Target 30 FPS

### Medium (Mobile/Entry-Level GPUs)
```rust
shadow_resolution: 1024
shadow_cascades: 3
enable_soft_shadows: true
lod_bias: 0.0
msaa_samples: 2
enable_post_processing: true
enable_bloom: true
enable_ssao: true (16 samples)
enable_ssr: false
enable_taa: false
max_particles: 5000
view_distance_multiplier: 0.75
target_fps: 60.0
```

**Focus**: Balanced performance and quality
- Basic post-processing enabled
- Moderate shadow quality
- Selective effects
- Target 60 FPS at 1080p

### High (Mid-Range GPUs)
```rust
shadow_resolution: 2048
shadow_cascades: 4
enable_soft_shadows: true
lod_bias: 0.3
msaa_samples: 4
enable_post_processing: true
enable_bloom: true
enable_ssao: true (32 samples)
enable_ssr: true
enable_taa: true
max_particles: 10000
view_distance_multiplier: 1.0
target_fps: 60.0
```

**Focus**: High visual quality with good performance
- Most effects enabled
- High-quality shadows
- TAA and SSR enabled
- Target 60 FPS at 1440p

### Ultra (High-End GPUs)
```rust
shadow_resolution: 4096
shadow_cascades: 4
enable_soft_shadows: true
lod_bias: 0.5
msaa_samples: 8
enable_post_processing: true
enable_bloom: true
enable_ssao: true (64 samples)
enable_ssr: true
enable_taa: true
max_particles: 50000
view_distance_multiplier: 1.5
target_fps: 60.0
```

**Focus**: Maximum visual quality
- All effects enabled at maximum quality
- Ultra shadow quality
- Extended view distance
- Target 60+ FPS at 4K

## Usage

### Basic Detection

```rust
use praxis_graphics::{HardwareTierDetector, HardwareTierConfig};
use vulkano::device::physical::PhysicalDevice;

// Detect hardware tier from physical device
let detector = HardwareTierDetector::from_physical_device(&physical_device);

// Get recommended preset
let preset = detector.recommended_preset();
println!("Recommended preset: {}", preset.name());

// Get configuration
let config = HardwareTierConfig::from_preset(preset);
```

### Applying Configuration

```rust
// Apply shadow settings
shadow_manager.set_resolution(config.shadow_resolution);
shadow_manager.set_cascade_count(config.shadow_cascades);
shadow_manager.set_soft_shadows(config.enable_soft_shadows);

// Apply LOD settings
lod_manager.set_global_lod_bias(config.lod_bias);
lod_manager.set_max_lod_level(config.max_lod_level);

// Apply post-processing settings
if config.enable_bloom {
    bloom_renderer.enable();
}
if config.enable_ssao {
    ssao_renderer.enable(config.ssao_samples);
}
if config.enable_ssr {
    ssr_renderer.enable();
}
if config.enable_taa {
    taa_renderer.enable();
}

// Apply optimization settings
optimization_config.set_gpu_culling(config.enable_gpu_culling);
optimization_config.set_hiz_occlusion(config.enable_hiz_occlusion);
optimization_config.set_mesh_streaming(config.enable_mesh_streaming);

// Configure adaptive quality
let adaptive_config = AdaptiveQualityConfig {
    target_fps: config.target_fps,
    min_shadow_resolution: config.shadow_resolution / 2,
    max_shadow_resolution: config.shadow_resolution,
    ..Default::default()
};
```

### Querying Hardware Details

```rust
// Get detailed hardware information
println!("{}", detector.summary());

// Check specific properties
println!("Vendor: {}", detector.vendor.name());
println!("VRAM: {} MB", detector.total_vram / (1024 * 1024));
println!("Tier: {}", detector.tier.name());

// Check feature support
if detector.supports_feature("ray_tracing") {
    println!("Ray tracing supported");
}
if detector.supports_feature("hiz_occlusion") {
    println!("Hi-Z occlusion recommended");
}

// Check device type
if detector.is_integrated() {
    println!("Integrated GPU - use power-efficient settings");
}
if detector.is_high_end() {
    println!("High-end GPU - maximize quality");
}
```

### Custom Tier Classification

You can also create custom configurations based on specific requirements:

```rust
use praxis_graphics::{HardwareTierConfig, QualityPreset};

// Start from a preset and customize
let mut config = HardwareTierConfig::from_preset(QualityPreset::High);

// Adjust specific settings
config.shadow_resolution = 1024;
config.enable_ssr = false;  // Disable SSR for performance
config.target_fps = 120.0;  // Target higher frame rate

// Or build from scratch
let custom_config = HardwareTierConfig {
    shadow_resolution: 2048,
    shadow_cascades: 3,
    enable_soft_shadows: true,
    lod_bias: 0.2,
    // ... other settings
    ..HardwareTierConfig::high()  // Use high preset as base
};
```

## Detection Algorithm

The tier determination uses the following heuristics:

1. **Device Type Check**
   - IntegratedGpu → Integrated tier (with exceptions)
   - Device name contains "integrated", "uhd", "iris" → Integrated tier
   - Device name contains "mobile", "laptop", "max-q" → Mobile tier

2. **Apple Silicon Exception**
   - Apple devices with 8+ GB VRAM → Mobile tier (despite being integrated)

3. **Discrete GPU Classification** (by VRAM)
   - ≤ 2 GB → Mobile tier
   - 3-6 GB → Mobile or Mid-Range (based on compute units)
   - 6-10 GB → Mid-Range tier
   - 12+ GB → High-End tier
   - 16+ GB with high compute units → High-End tier (confirmed)

4. **Vendor-Specific Notes**
   - **NVIDIA**: Excellent Vulkan support, strong compute performance
   - **AMD**: Excellent Vulkan support, good async compute
   - **Intel**: Focus on efficiency, improving driver support
   - **Apple**: Unified memory architecture, consider Metal for best performance

## Integration with Adaptive Quality

The hardware tier detection works seamlessly with the adaptive quality system:

```rust
use praxis_graphics::{
    HardwareTierDetector, AdaptiveQualitySystem, AdaptiveQualityConfig
};

// Detect hardware and get config
let detector = HardwareTierDetector::from_physical_device(&physical_device);
let tier_config = detector.recommended_config();

// Configure adaptive quality with tier-appropriate settings
let adaptive_config = AdaptiveQualityConfig {
    target_fps: tier_config.target_fps,
    min_shadow_resolution: tier_config.shadow_resolution / 2,
    max_shadow_resolution: tier_config.shadow_resolution,
    initial_shadow_resolution: tier_config.shadow_resolution,
    min_lod_bias: tier_config.lod_bias - 0.5,
    max_lod_bias: tier_config.lod_bias + 0.5,
    initial_lod_bias: tier_config.lod_bias,
    enable_shadow_resolution_adjustment: true,
    ..Default::default()
};

let mut adaptive_quality = AdaptiveQualitySystem::new(adaptive_config);

// In render loop
adaptive_quality.update(frame_time);
```

## Serialization

Both `HardwareTierConfig` and related types support serialization:

```rust
use serde_json;

// Save configuration
let config = detector.recommended_config();
let json = serde_json::to_string_pretty(&config)?;
std::fs::write("quality_settings.json", json)?;

// Load configuration
let json = std::fs::read_to_string("quality_settings.json")?;
let config: HardwareTierConfig = serde_json::from_str(&json)?;
```

## Example Demo

Run the hardware tier detection demo to see the system in action:

```bash
cargo run --example hardware_tier_demo
```

The demo will:
1. Detect your GPU and display detailed hardware information
2. Show the recommended quality preset and configuration
3. Display feature support based on your hardware tier
4. Provide tier-specific optimization recommendations
5. Compare all quality presets side-by-side
6. Show vendor-specific notes and best practices
7. Render a simple scene using the detected settings

## Best Practices

1. **Run Detection Once**: Perform hardware detection during application startup
2. **User Override**: Allow users to manually select quality presets
3. **Save Settings**: Persist user-selected or auto-detected settings
4. **Validate**: Always validate configurations before applying
5. **Combine Systems**: Use hardware tier detection with adaptive quality for optimal results
6. **Profile**: Test presets on target hardware to verify performance
7. **Gradual Changes**: When applying new settings, fade or transition smoothly

## Educational Value

This system demonstrates:
- GPU capability detection via Vulkan API
- Heuristic-based hardware classification
- Preset-based configuration management
- Vendor-specific optimization strategies
- Integration patterns for quality management
- Serialization for settings persistence
