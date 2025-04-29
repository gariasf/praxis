# Praxis Game Engine

## Global Rules
- Prioritize free/open, battle-proven libraries only
- Avoid proprietary or costly tools
- Target Vulkan exclusively; retro-compatibility with old graphics APIs only if strictly necessary
- First aim for a working prototype; performance optimization and refinements come later
- Enforce clear coding guidelines, separation of concerns, and descriptive naming
- Use a step-by-step approach, keep initial scope minimal
- Provide a learning path in the roadmap
- Simple over complex
- No unnecessary abstractions

## Project Overview

Praxis is a modern 3D game engine built using C++20 and Vulkan designed to serve as a foundation for game development with a focus on performance, flexibility, and ease of use. The engine aims to provide a robust framework for game developers to create high-quality 3D experiences without the licensing constraints of commercial engines.

### Goals
- Create a cross-platform game engine using modern C++ practices
- Provide a comprehensive Vulkan-based rendering pipeline
- Establish a flexible architecture that can be extended for various game genres
- Develop a modular system that allows developers to use only what they need
- Eventually support open-world RPG-style games with complex scenes and interactions

### Success Criteria
- Engine can render complex 3D scenes with modern lighting techniques
- Physics simulation supports realistic interactions
- Input handling works across multiple device types
- Asset pipeline supports industry-standard formats
- Performance is competitive with commercial engines for similar workloads
- Eventually can run open-world RPGs with complexity similar to games like Skyrim

## Technical Scope

### Language and Toolchain
- C++20 standard (or later)
- Recommended compilers:
  - MSVC (Visual Studio 2022) for Windows
  - GCC 10+ or Clang 12+ for Linux
  - Clang for macOS
- Build system: CMake 3.20+

### SDL 3.2.10 Integration
- Window creation and management
- Input handling (keyboard, mouse, gamepad)
- Audio system foundation
- Cross-platform support

### Vulkan Implementation
- Instance and device initialization
- Swapchain management
- Command buffers and synchronization
- Pipeline creation
- Shader compilation and management
- Render pass organization

## Library Recommendations

- **SDL 3.2.10**: Core windowing, input, and platform abstraction library
- **Vulkan SDK 1.3+**: Graphics API providing modern GPU acceleration
- **GLM**: Mathematics library specifically designed for graphics programming
- **stb_image**: Lightweight image loading for textures without complex dependencies
- **Dear ImGui**: Immediate-mode GUI for debugging and tools
- **spdlog**: Fast, thread-safe logging library for diagnostics
- **EnTT**: Fast, modern entity-component-system for game object management
- **nlohmann/json**: JSON parser for configuration and data storage
- **assimp**: Open asset import library for loading 3D models and scenes
- **PhysX**: Open-source physics engine (now Apache 2.0 licensed)
- **OpenAL-Soft**: Open-source audio library for 3D sound
- **{fmt}**: Modern formatting library for strings
- **backward-cpp**: Stack trace library for error handling
- **Catch2**: Unit testing framework

## Coding Guidelines

### Directory Layout
```
praxis/
├── assets/                  # Default assets for testing
├── build/                   # Build outputs (should be in .gitignore)
├── cmake/                   # CMake modules
├── docs/                    # Documentation
├── examples/                # Example applications
├── external/                # Third-party dependencies
├── include/                 # Public headers
│   └── praxis/              # Engine headers
│       ├── core/            # Core systems
│       ├── graphics/        # Rendering code
│       ├── audio/           # Audio systems
│       ├── physics/         # Physics simulation
│       ├── input/           # Input handling
│       ├── scene/           # Scene management
│       └── utils/           # Utility functions
├── src/                     # Source files
│   └── [mirrors include structure]
├── tests/                   # Unit and integration tests
└── tools/                   # Development tools
```

### Module Responsibilities

- **Core**: Engine initialization, memory management, threading, event system
- **Graphics**: Rendering pipeline, material system, shader management
- **Audio**: Sound playback, 3D audio, music streaming
- **Physics**: Collision detection, rigid body dynamics, constraints
- **Input**: Device abstraction, input mapping, context-sensitive controls
- **Scene**: Entity management, scene graph, serialization
- **Utils**: Math helpers, data structures, file I/O, logging

### Naming Conventions

- **Files**: Snake case for implementation files, pascal case for headers
  - E.g., `vulkan_renderer.cpp`, `VulkanRenderer.h`
- **Classes**: Pascal case
  - E.g., `class RenderPipeline`
- **Methods/Functions**: Camel case
  - E.g., `void initializeRenderer()`
- **Variables**: Camel case
  - E.g., `float deltaTime`
- **Member Variables**: Camel case with 'm_' prefix
  - E.g., `m_currentScene`
- **Constants/Enums**: All caps with underscores
  - E.g., `MAX_LIGHTS`, `enum class RenderMode { FORWARD, DEFERRED }`
- **Namespaces**: Lower case
  - E.g., `namespace praxis::graphics`

### Build System

- CMake-based build system
- Support for multi-platform builds
- Package management via CMake FetchContent or Git submodules
- Modular build targets to allow selective inclusion of engine features
- Separate build configurations for Debug, Release, RelWithDebInfo
- Unit tests integrated with CTest

## Roadmap / TODO List

### 1. [x] Project Setup and Build System 📗
- [x] Set up basic directory structure
- [x] Create initial CMake configuration
- [x] Add core external dependencies
- [x] Create basic abstraction headers
- **Learning**: CMake in Practice (book), [CMake Tutorial](https://cmake.org/cmake/help/latest/guide/tutorial/index.html)

### 2. [ ] Window Creation and Basic Vulkan Setup 📙
- [x] Initialize SDL window
- [x] Create Vulkan instance
- [x] Set up validation layers
- [x] Create device and queues
- [x] Initialize swapchain
- [x] Basic render pass setup
- [ ] Implement proper error recovery
- [ ] Add device lost handling
- [ ] Improve swapchain recreation robustness
- [ ] Add validation layer message filtering
- [ ] Add debug utils extension support
- [ ] Implement device feature queries
- **Learning**: 
  - [Vulkan Tutorial](https://vulkan-tutorial.com/)
  - "Game Engine Architecture" Ch. 15 (Graphics Engine Low-Level)
  - "Real-Time Rendering" Ch. 1 (Graphics Pipeline Overview)

### 3. [ ] Core Graphics Pipeline Implementation 📘
- [ ] Graphics pipeline setup
- [ ] Shader management system
  - [ ] SPIR-V compilation
  - [ ] Runtime shader hot-reload
  - [ ] Shader reflection
- [ ] Vertex/Index buffer management
- [ ] Uniform buffer system
- [ ] Descriptor set/layout handling
- [ ] Material system foundation
- [ ] Pipeline state objects
- [ ] Pipeline layout management
- [ ] Push constant optimization
- [ ] Pipeline derivatives
- [ ] Pipeline cache serialization
- **Learning**: 
  - [Vulkan Shader Resource Binding](https://www.khronos.org/blog/vulkan-shader-resource-binding)
  - "Real-Time Rendering" Ch. 2-3 (GPU Architecture, Graphics Pipeline)
  - "Game Engine Architecture" Ch. 15.4 (Graphics Resource Management)
  - "Foundations of Game Engine Development, Vol. 2" Ch. 1-2 (Rendering Pipeline, Shader Programs)

### 4. [ ] Advanced Rendering Features 📕
- [ ] Depth buffer implementation
- [ ] Multisampling (MSAA) support
- [ ] Texture loading and management
- [ ] Multiple render targets (MRT)
- [ ] Pipeline caching
- [ ] Command buffer optimization
- [ ] Modern Vulkan features
  - [ ] Dynamic state
  - [ ] Timeline semaphores
  - [ ] Buffer device address
- [ ] Bindless rendering setup
- [ ] Mesh shaders integration
- [ ] Ray tracing foundation
- [ ] GPU-driven rendering pipeline
- **Learning**: 
  - "Real-Time Rendering" Ch. 5 (Shading Basics), Ch. 9 (Pipeline Optimization)
  - "Physically Based Rendering" Ch. 7 (Sampling and Reconstruction)
  - "Foundations of Game Engine Development, Vol. 2" Ch. 4 (Advanced Rendering)

### 5. [ ] Memory and Resource Management 📗
- [ ] Vulkan Memory Allocator (VMA) integration
- [ ] Staging buffer system
- [ ] Resource pooling
- [ ] Texture streaming
- [ ] Buffer defragmentation
- [ ] Resource lifetime management
- [ ] Memory budget tracking
- [ ] Resource residency management
- [ ] Memory defragmentation strategies
- [ ] Resource aliasing
- **Learning**: 
  - "Game Engine Architecture" Ch. 5.3 (Memory Management)
  - "Foundations of Game Engine Development, Vol. 1" Ch. 2 (Memory Management)
  - [Vulkan Memory Management](https://gpuopen.com/learn/vulkan-memory-management/)

### 6. [ ] Performance and Debug Tools 📙
- [ ] Debug markers and labels
- [ ] Performance markers
- [ ] GPU timestamp queries
- [ ] Memory leak detection
- [ ] Pipeline statistics
- [ ] Frame capture support
- [ ] Resource tracking
- [ ] Shader debugging support
- [ ] RenderDoc integration
- [ ] Performance counter tracking
- [ ] Automated performance testing
- **Learning**: 
  - "Game Engine Architecture" Ch. 5.4 (Debug Systems)
  - "Real-Time Rendering" Ch. 23 (Graphics Pipeline Performance)

### 7. [ ] Scene Management and Rendering 📘
- [ ] Scene graph implementation
- [ ] Frustum culling
- [ ] LOD system
- [ ] Instanced rendering
- [ ] Indirect drawing
- [ ] Occlusion culling
- [ ] View frustum optimization
- [ ] Portal culling system
- [ ] Dynamic batching system
- [ ] Instance data management
- **Learning**: 
  - "Real-Time Rendering" Ch. 19 (Acceleration Algorithms)
  - "Game Engine Architecture" Ch. 14 (Scene Graph/Culling Optimizations)
  - "Foundations of Game Engine Development, Vol. 2" Ch. 3 (Scene Management)

### 8. [ ] Multi-threading Support 📕
- [ ] Command buffer multi-threading
- [ ] Resource loading thread
- [ ] Async compute
- [ ] Job system integration
- [ ] Thread pool management
- [ ] Command buffer recording parallelization
- [ ] Pipeline state creation threading
- [ ] Resource upload threading
- [ ] Parallel frustum culling
- **Learning**: 
  - "Game Engine Architecture" Ch. 7 (Multi-threading)
  - "Foundations of Game Engine Development, Vol. 1" Ch. 4 (Parallel Algorithms)

### 9. [ ] Advanced Scene Management 📘
- [ ] Spatial partitioning system
  - [ ] Octree/quadtree implementation
  - [ ] Dynamic scene subdivision
  - [ ] Visibility determination
- [ ] World streaming system
  - [ ] Chunk-based loading/unloading
  - [ ] Distance-based LOD management
  - [ ] Async resource streaming
- [ ] Scene serialization
  - [ ] Binary scene format
  - [ ] Delta compression for saves
  - [ ] Scene diffing and patching
- [ ] Scene editor tools
  - [ ] WYSIWYG editor integration
  - [ ] Scene hierarchy manipulation
  - [ ] Property editors
- [ ] Hierarchical Z-buffer occlusion
- [ ] Dynamic object management
- [ ] Scene graph optimization
- **Learning**: 
  - "Game Engine Architecture" Ch. 14.7 (Scene Graph Design and Implementation)
  - "Foundations of Game Engine Development, Vol. 2" Ch. 3 (Scene Graph Systems)

### 10. [ ] Advanced Graphics Features 📕
- [ ] Advanced lighting system
  - [ ] Dynamic time of day
  - [ ] Global illumination
  - [ ] Volumetric lighting
  - [ ] Dynamic weather effects
- [ ] Post-processing pipeline
  - [ ] HDR rendering
  - [ ] Tone mapping
  - [ ] Bloom and eye adaptation
  - [ ] Screen-space effects (SSAO, SSR)
- [ ] Terrain system
  - [ ] Height-field terrain
  - [ ] Dynamic tessellation
  - [ ] Terrain LOD
  - [ ] Foliage system
- [ ] Advanced materials
  - [ ] PBR workflow
  - [ ] Material layering
  - [ ] Decal system
  - [ ] Dynamic surface effects (wetness, snow)
- [ ] Clustered forward/deferred rendering
- [ ] Tiled light management
- [ ] Shadow mapping techniques
  - [ ] Cascaded shadow maps
  - [ ] Variance shadow maps
  - [ ] Moment shadow maps
- **Learning**: 
  - "Real-Time Rendering" Ch. 7-8 (Shadows), Ch. 10-11 (Local and Global Illumination)
  - "Physically Based Rendering" Ch. 8 (Reflection Models), Ch. 11 (Volume Rendering)
  - "Foundations of Game Engine Development, Vol. 2" Ch. 5 (Lighting and Materials)

### 11. [ ] Animation and Characters 📗
- [ ] Animation system
  - [ ] Skeletal animation
  - [ ] Animation blending
  - [ ] Inverse kinematics
  - [ ] Ragdoll physics
- [ ] Character system
  - [ ] Character customization
  - [ ] Equipment system
  - [ ] Dynamic cloth simulation
  - [ ] Hair/fur rendering
- [ ] Crowd system
  - [ ] Instanced character rendering
  - [ ] LOD for animated characters
  - [ ] Animation compression
- [ ] GPU-driven skinning
- [ ] Animation compression
- [ ] Motion matching system
- **Learning**: 
  - "Game Engine Architecture" Ch. 11 (Animation Systems)
  - "Real-Time Rendering" Ch. 4 (Animation and Skinning)

### 12. [ ] Game Systems Integration 📙
- [ ] Quest system
  - [ ] Quest state management
  - [ ] Trigger system
  - [ ] Dialog system integration
- [ ] Inventory system
  - [ ] Item database
  - [ ] Equipment management
  - [ ] Crafting system
- [ ] AI and NPC system
  - [ ] Behavior trees
  - [ ] Pathfinding
  - [ ] Dynamic NPC scheduling
  - [ ] Combat AI
- [ ] Save/Load system
  - [ ] State serialization
  - [ ] Save file management
  - [ ] Backwards compatibility
- [ ] Event system optimization
- [ ] Component serialization
- [ ] System dependencies management
- **Learning**: 
  - "Game Engine Architecture" Ch. 6 (Game Loop/Update), Ch. 13 (Runtime Gameplay Systems)

### 13. [ ] Performance Optimization for Open Worlds 📕
- [ ] Memory management
  - [ ] Custom allocators for game systems
  - [ ] Memory defragmentation
  - [ ] Resource streaming optimization
- [ ] CPU optimization
  - [ ] Job system for game logic
  - [ ] SIMD optimizations
  - [ ] Cache-friendly data structures
- [ ] GPU optimization
  - [ ] Draw call batching
  - [ ] GPU-driven rendering
  - [ ] Mesh shader pipeline
  - [ ] Compute shader utilization
- [ ] Loading optimization
  - [ ] Background loading
  - [ ] Asset compression
  - [ ] Load time profiling
- [ ] Streaming distance optimization
- [ ] Memory residency prediction
- [ ] Dynamic quality scaling
- **Learning**: 
  - "Real-Time Rendering" Ch. 21 (Acceleration Algorithms), Ch. 23 (Graphics Pipeline Performance)
  - "Game Engine Architecture" Ch. 7 (High-Level Engine Systems)
  - "Foundations of Game Engine Development, Vol. 1" Ch. 5 (Performance and Optimization)

### 14. [ ] Tools and Pipeline 📘
- [ ] Asset pipeline
  - [ ] Asset preprocessing
  - [ ] Asset versioning
  - [ ] Hot reload system
- [ ] Development tools
  - [ ] Scene editor
  - [ ] Material editor
  - [ ] Particle editor
  - [ ] Animation editor
- [ ] Debugging tools
  - [ ] Performance profiler
  - [ ] Memory tracker
  - [ ] Scene inspector
  - [ ] State debugger
- [ ] Build pipeline
  - [ ] Asset packaging
  - [ ] Dependency tracking
  - [ ] Distribution preparation
- [ ] Asset dependency tracking
- [ ] Incremental build system
- [ ] Runtime performance analysis
- **Learning**: 
  - "Game Engine Architecture" Ch. 16 (Tools and Asset Pipeline)
  - "Physically Based Rendering" Ch. 6 (Texture and Materials Pipeline)

## Progress Monitoring and Review

### Review Cadence
- Weekly progress checks for each incremental feature
- Monthly architecture review to ensure coherence and maintainability
- Quarterly roadmap reassessment to adjust priorities based on progress

### Progress Metrics
- Feature completion rate against roadmap
- Code quality (measured by static analysis tools)
- Performance benchmarks compared to initial baseline
- Documentation coverage
- Unit test coverage

### Learning Resources Collection
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Sascha Willems Vulkan Samples](https://github.com/SaschaWillems/Vulkan)
- [Vulkan Cookbook](https://www.packtpub.com/product/vulkan-cookbook/9781786468154)
- [VulkanGuide.dev](https://vkguide.dev/)
- "Game Engine Architecture" by Jason Gregory
- "Real-Time Rendering" by Tomas Akenine-Möller
- "3D Game Engine Design" by David H. Eberly
- [Khronos Vulkan Samples](https://github.com/KhronosGroup/Vulkan-Samples)
- [LearnOpenGL](https://learnopengl.com/) (for general graphics concepts)
- [Physically Based Rendering](https://www.pbr-book.org/) 