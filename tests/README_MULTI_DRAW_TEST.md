# Multi-Draw Indirect Rendering Test

## Overview

The `multi_draw_indirect_test.rs` integration test validates the GPU culling and multi-draw indirect rendering system in the Praxis engine. It creates 200+ objects with 10 different materials and verifies:

1. **Batch Reduction**: Draw calls are efficiently batched by material (250 draws → ~10 batches = 25x reduction)
2. **Indirect Draw Buffer**: GPU-generated indirect draw commands are valid and correct
3. **Frustum Culling**: Objects outside the view frustum are properly culled
4. **Buffer Content**: Visible indices and draw counts are accurate

## Test Coverage

### GPU Integration Tests (Require Vulkan + CMake)
- `test_multi_draw_indirect_batch_reduction` - Verifies 25x batch reduction with 250 objects, 10 materials
- `test_indirect_draw_buffer_validation` - Validates indirect draw command structure and content
- `test_visible_indices_buffer` - Verifies visible object indices are valid and unique
- `test_frustum_culling_accuracy` - Tests culling of objects outside view frustum
- `test_draw_count_buffer` - Verifies draw count buffer updates correctly

### Unit Tests (No GPU Required)
- `test_gpu_draw_command_creation` - Tests GPU draw command data structure
- `test_mesh_data_creation` - Tests mesh metadata structure
- `test_indirect_draw_command_layout` - Verifies 20-byte VkDrawIndexedIndirectCommand layout
- `test_frustum_plane_extraction` - Tests frustum plane extraction from view-projection matrix
- `test_create_many_draw_commands` - Tests creation of 250 draw commands with material distribution
- `test_create_many_mesh_data` - Tests creation of 200 mesh data entries
- `test_batch_reduction_calculation` - Verifies 25x batch reduction math
- `test_gpu_draw_command_size` - Validates 96-byte GPU buffer alignment
- `test_gpu_mesh_data_size` - Validates 16-byte GPU buffer alignment

## Requirements

### For GPU Integration Tests

1. **Vulkan-capable GPU and drivers**
   - Compute shader support required
   - Run `vulkaninfo` to verify your setup

2. **CMake** (for shader compilation via vulkano-shaders)
   - **Windows**: `winget install Kitware.CMake`
     - Or download from: https://cmake.org/download/
     - Add to PATH: `C:\Program Files\CMake\bin`
   - **Linux**: `sudo apt install cmake` (Ubuntu/Debian) or `sudo dnf install cmake` (Fedora)
   - **macOS**: `brew install cmake`

3. **Verify CMake**: Run `cmake --version` to confirm installation

### For Unit Tests Only

Unit tests don't require Vulkan or CMake - they test data structures and math only.

## Running Tests

### Run All Tests (GPU + Unit)
```bash
cargo test --test multi_draw_indirect_test -- --nocapture --test-threads=1
```

### Run Only Unit Tests (No GPU/CMake Required)
```bash
# Run specific unit tests by name pattern
cargo test --test multi_draw_indirect_test test_gpu_draw_command_creation -- --nocapture
cargo test --test multi_draw_indirect_test test_mesh_data_creation -- --nocapture
cargo test --test multi_draw_indirect_test test_batch_reduction_calculation -- --nocapture
```

### Run Only GPU Tests
```bash
cargo test --test multi_draw_indirect_test test_multi_draw_indirect_batch_reduction -- --nocapture
cargo test --test multi_draw_indirect_test test_indirect_draw_buffer_validation -- --nocapture
cargo test --test multi_draw_indirect_test test_frustum_culling_accuracy -- --nocapture
```

## Test Output Example

```
test test_multi_draw_indirect_batch_reduction ... ok
  ✓ Batch reduction verified: 10 materials batch 250 objects

test test_indirect_draw_buffer_validation ... ok
  ✓ All 50 indirect draw commands are valid

test test_frustum_culling_accuracy ... ok
  ✓ Frustum culling reduced object count from 200 to 85

test test_batch_reduction_calculation ... ok
  Batch reduction: 250 objects with 10 materials = 25x reduction
```

## Expected Performance

With 250 objects and 10 materials:
- **Without batching**: 250 individual draw calls
- **With multi-draw indirect**: ~10 batches (one per material)
- **Reduction factor**: 25x fewer API calls

This dramatic reduction in draw call overhead enables efficient rendering of large scenes with thousands of objects.

## Troubleshooting

### "couldn't find required command: cmake"
Install CMake as described in Requirements section above.

### "No suitable physical device found"
Your system doesn't have a Vulkan-capable GPU or drivers. Install latest GPU drivers or run on a system with Vulkan support.

### "Failed to create Vulkan instance"
Vulkan drivers not installed. Install Vulkan SDK or GPU vendor drivers with Vulkan support.

## Implementation Details

The test uses:
- `GpuCullingManager` for compute shader-based frustum culling
- `GpuDrawCommand` structs with bounding spheres and transform matrices
- `IndirectDrawCommand` buffers generated on GPU
- Frustum plane extraction from view-projection matrices
- Material-based batching for efficient rendering

See `crates/praxis_graphics/src/gpu_culling.rs` for implementation details.
