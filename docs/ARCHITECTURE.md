# Praxis Game Engine Architecture

## Overview
Praxis is a modular, cross-platform 3D game engine built in C++20, designed for performance, extensibility, and clarity. The engine is organized into logical modules, each with a clear responsibility, and follows modern C++ and software engineering best practices.

## Directory Structure
```
praxis/
├── assets/      # Default assets for testing
├── build/       # Build outputs (ignored in VCS)
├── cmake/       # CMake modules and scripts
├── docs/        # Documentation (including this file)
├── examples/    # Example applications
├── external/    # Third-party dependencies
├── include/     # Public headers (mirrors src/)
│   └── praxis/
│       ├── core/
│       ├── graphics/
│       ├── audio/
│       ├── physics/
│       ├── input/
│       ├── scene/
│       └── utils/
├── src/         # Source files (mirrors include/)
├── tests/       # Unit and integration tests (mirrors modules)
└── tools/       # Development tools
```

## Module Responsibilities
- **core**: Engine initialization, memory management, threading, event system
- **graphics**: Rendering pipeline, Vulkan integration, material and shader management
- **audio**: Sound playback, 3D audio, music streaming
- **physics**: Collision detection, rigid body dynamics, constraints
- **input**: Device abstraction, input mapping, context-sensitive controls
- **scene**: Entity management, scene graph, serialization
- **utils**: Math helpers, data structures, file I/O, logging

## Build System
- Uses CMake (3.20+) as the build system, with Ninja as the preferred generator
- Supports multi-platform builds (Windows, Linux, macOS)
- Dependencies managed via CMake FetchContent or Git submodules
- Modular build targets allow selective inclusion of engine features
- Separate build configurations: Debug, Release, RelWithDebInfo
- Unit tests integrated with CTest
- Clang-Tidy and Clang-Format are enabled for code analysis and style enforcement

## Testing Approach
- Strict test-driven development (TDD): tests are written before implementation
- Catch2 is used for all unit and integration tests
- Target at least 80% test coverage for all modules
- All new features and bug fixes must include corresponding tests
- Tests are organized in the `tests/` directory, mirroring the engine's module structure
- Tests are integrated with CTest and the build system

## Documentation
- Doxygen-style documentation is required for all public APIs, non-obvious code, and new features
- Architectural documentation is required for new modules and major infrastructure changes
- Documentation is kept up to date with code changes and provided in the `docs/` directory

## Coding Style
- Follows modern C++23 best practices
- Enforces clear naming conventions, modularity, and code readability
- See `.cursor/rules/` for detailed coding, testing, and documentation rules

## Extensibility
- The engine is designed to be modular and extensible
- New modules should follow the established directory and namespace conventions
- All new features must be documented and tested 