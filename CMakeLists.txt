cmake_minimum_required(VERSION 3.20)
project(Praxis VERSION 0.1.0 LANGUAGES CXX)

# C++23 required
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build options
option(PRAXIS_BUILD_TESTS "Build tests" ON)
option(PRAXIS_BUILD_EXAMPLES "Build examples" ON)
option(PRAXIS_BUILD_DOCS "Build documentation" OFF)
option(PRAXIS_USE_VULKAN_VALIDATION "Enable Vulkan validation layers" ON)

# Global compiler options
if(MSVC)
  # MSVC
  add_compile_options(/W4 /MP)
else()
  # GCC / CLANG
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Output directories
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Dependencies
include(FetchContent)

# SDL3
FetchContent_Declare(
  SDL
  GIT_REPOSITORY https://github.com/libsdl-org/SDL.git
  GIT_TAG release-3.2.10
)
FetchContent_MakeAvailable(SDL)

# GLM
FetchContent_Declare(
  glm
  GIT_REPOSITORY https://github.com/g-truc/glm.git
  GIT_TAG 1.0.1
)
FetchContent_MakeAvailable(glm)

# spdlog
FetchContent_Declare(
  spdlog
  GIT_REPOSITORY https://github.com/gabime/spdlog.git
  GIT_TAG v1.15.2
)
FetchContent_MakeAvailable(spdlog)

# fmt
FetchContent_Declare(
  fmt
  GIT_REPOSITORY https://github.com/fmtlib/fmt.git
  GIT_TAG 11.1.4
)
FetchContent_MakeAvailable(fmt)

# EnTT (ECS)
FetchContent_Declare(
  entt
  GIT_REPOSITORY https://github.com/skypjack/entt.git
  GIT_TAG v3.15.0
)
FetchContent_MakeAvailable(entt)

# Vulkan headers
find_package(Vulkan REQUIRED)

# Create include directories if they don't exist
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/praxis)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/src)

# Add engine core library
add_library(praxis_core STATIC
  # Core system
  src/core/Engine.cpp
  
  # Graphics system
  src/graphics/Renderer.cpp
  
  # Utility classes
  src/utils/Logger.cpp
)

# Include directories
target_include_directories(praxis_core
  PUBLIC 
    ${CMAKE_CURRENT_SOURCE_DIR}/include
  PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# Link dependencies
target_link_libraries(praxis_core
  PUBLIC
    SDL3::SDL3
    glm::glm
    spdlog::spdlog
    fmt::fmt
    EnTT::EnTT
    Vulkan::Vulkan
)

# Define preprocessor macros
if(PRAXIS_USE_VULKAN_VALIDATION)
  target_compile_definitions(praxis_core PRIVATE PRAXIS_VULKAN_VALIDATION)
endif()

# Build examples if enabled
if(PRAXIS_BUILD_EXAMPLES)
  add_subdirectory(examples)
endif()

# Build tests if enabled
if(PRAXIS_BUILD_TESTS)
  enable_testing()
  add_subdirectory(tests)
endif()

# Build docs if enabled
if(PRAXIS_BUILD_DOCS)
  add_subdirectory(docs)
endif()

# Install rules
install(TARGETS praxis_core
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
  RUNTIME DESTINATION bin
  INCLUDES DESTINATION include
)

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/praxis
  DESTINATION include
)

# Create base directory structure
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/assets)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/docs)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/examples)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/external)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tests)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tools)
file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/cmake)