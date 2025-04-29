# Praxis Game Engine

Disclaimer: A learning project. Development and topic study is assisted by AI.

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

Praxis is a modern 3D game engine built using C++23 and Vulkan designed to serve as a foundation for game development with a focus on performance, flexibility, and ease of use. The engine aims to provide a robust framework to create high-quality 3D experiences without the licensing constraints of commercial engines.

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

## Development Roadmap

The detailed development roadmap is maintained in [ROADMAP.md](ROADMAP.md). The roadmap outlines:
- Major milestones and their dependencies
- Priority levels for different features
- Progress metrics and success criteria
- Learning resources for each development phase

The roadmap is designed to support incremental learning and development, with each section building upon previous work. It's structured to be manageable over a couple of years of part-time development.

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

## Learning Resources Collection

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