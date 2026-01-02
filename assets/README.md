# Assets Directory

This directory contains asset files used by the Praxis engine examples and tests.

## Directory Structure

- `models/` - 3D model files (OBJ, etc.)
- `textures/` - Texture image files (PNG, JPEG, etc.) - currently empty as the comprehensive_scene_demo generates textures procedurally

## Models

### cube.obj

A simple cube mesh used for testing the OBJ loader. Contains:
- 8 vertices
- 6 face normals
- 12 triangles (2 per face)

This file is used by the `obj_loader_demo` example.

## Adding New Assets

When adding new assets:

1. Place them in the appropriate subdirectory
2. Update this README with a description
3. Keep file sizes reasonable (< 1MB for test assets)
4. Use common formats (OBJ for models, PNG for textures, etc.)
5. Include proper attribution if using third-party assets
