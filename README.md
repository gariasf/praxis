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

### 1. [ ] Project Setup and Build System 📗
- Set up basic directory structure
- Create initial CMake configuration
- Add core external dependencies
- Create basic abstraction headers
- **Learning**: CMake in Practice (book), [CMake Tutorial](https://cmake.org/cmake/help/latest/guide/tutorial/index.html)
```cmake
cmake_minimum_required(VERSION 3.20)
project(Praxis VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Options
option(PRAXIS_BUILD_TESTS "Build tests" ON)
option(PRAXIS_BUILD_EXAMPLES "Build examples" ON)

# Dependencies
include(FetchContent)
FetchContent_Declare(
  SDL
  GIT_REPOSITORY https://github.com/libsdl-org/SDL.git
  GIT_TAG release-3.2.0
)
FetchContent_MakeAvailable(SDL)

# Add core library
add_library(praxis_core
  src/core/engine.cpp
  src/core/logger.cpp
)

# Include directories
target_include_directories(praxis_core
  PUBLIC 
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Link dependencies
target_link_libraries(praxis_core
  PUBLIC
    SDL3::SDL3
)
```

### 2. [ ] Window Creation and Vulkan Setup 📙
- Initialize SDL window
- Create Vulkan instance
- Set up validation layers
- Create device and queues
- Initialize swapchain
- **Learning**: [Vulkan Tutorial](https://vulkan-tutorial.com/), "Vulkan Programming Guide" by Graham Sellers
```cpp
bool Engine::initialize() {
  // Initialize SDL
  if (SDL_Init(SDL_INIT_VIDEO) < 0) {
    m_logger.error("SDL initialization failed: {}", SDL_GetError());
    return false;
  }
  
  // Create window
  m_window = SDL_CreateWindow(
    "Praxis Engine",
    SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
    1280, 720,
    SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE
  );
  
  if (!m_window) {
    m_logger.error("Window creation failed: {}", SDL_GetError());
    return false;
  }
  
  // Initialize Vulkan
  if (!m_renderer.initialize(m_window)) {
    m_logger.error("Renderer initialization failed");
    return false;
  }
  
  return true;
}
```

### 3. [ ] Simple Render Loop and Triangle 📙
- Create command buffers
- Set up render passes
- Implement basic shader system
- Draw a triangle on screen
- Handle window events
- **Learning**: [Sascha Willems Vulkan Samples](https://github.com/SaschaWillems/Vulkan), "Vulkan Cookbook" by Pawel Lapinski
```cpp
void Renderer::render() {
  // Wait for fence to ensure previous frame is done
  vkWaitForFences(m_device, 1, &m_inFlightFences[m_currentFrame], VK_TRUE, UINT64_MAX);
  
  // Acquire next image
  uint32_t imageIndex;
  VkResult result = vkAcquireNextImageKHR(m_device, m_swapchain, UINT64_MAX, 
                                         m_imageAvailableSemaphores[m_currentFrame], 
                                         VK_NULL_HANDLE, &imageIndex);
  
  // Record command buffer
  VkCommandBuffer cmdBuffer = m_commandBuffers[m_currentFrame];
  recordCommandBuffer(cmdBuffer, imageIndex);
  
  // Submit command buffer
  VkSubmitInfo submitInfo{};
  submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
  // ... set up semaphores and command buffer info
  
  vkResetFences(m_device, 1, &m_inFlightFences[m_currentFrame]);
  vkQueueSubmit(m_graphicsQueue, 1, &submitInfo, m_inFlightFences[m_currentFrame]);
  
  // Present
  VkPresentInfoKHR presentInfo{};
  presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
  // ... set up present info
  
  vkQueuePresentKHR(m_presentQueue, &presentInfo);
  
  m_currentFrame = (m_currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}
```

### 4. [ ] Math Library Integration 📗
- Integrate GLM
- Create math utility wrapper
- Implement transformation helpers
- Set up camera projection matrices
- **Learning**: [GLM Documentation](https://github.com/g-truc/glm/blob/master/manual.md), 3D Math Primer for Graphics and Game Development
```cpp
namespace praxis::math {

class Transform {
public:
  Transform() : m_position(0.0f), m_rotation(1.0f, 0.0f, 0.0f, 0.0f), m_scale(1.0f) {}
  
  void setPosition(const glm::vec3& position) { m_position = position; }
  void setRotation(const glm::quat& rotation) { m_rotation = rotation; }
  void setScale(const glm::vec3& scale) { m_scale = scale; }
  
  glm::mat4 getMatrix() const {
    glm::mat4 translation = glm::translate(glm::mat4(1.0f), m_position);
    glm::mat4 rotation = glm::mat4_cast(m_rotation);
    glm::mat4 scale = glm::scale(glm::mat4(1.0f), m_scale);
    
    return translation * rotation * scale;
  }
  
private:
  glm::vec3 m_position;
  glm::quat m_rotation;
  glm::vec3 m_scale;
};

}
```

### 5. [ ] Asset Loading 📙
- Set up stb_image for texture loading
- Implement texture abstractions
- Create mesh data structures
- Add simple OBJ or glTF loader
- **Learning**: [Asset Loading with Vulkan](https://vkguide.dev/docs/chapter-3/), stb_image documentation
```cpp
bool TextureLoader::loadFromFile(const std::string& filename, Texture& outTexture) {
  int width, height, channels;
  unsigned char* data = stbi_load(filename.c_str(), &width, &height, &channels, STBI_rgb_alpha);
  
  if (!data) {
    m_logger.error("Failed to load texture: {}", filename);
    return false;
  }
  
  VkDeviceSize imageSize = width * height * 4;
  
  // Create staging buffer
  Buffer stagingBuffer;
  m_bufferManager.createBuffer(
    imageSize,
    VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
    VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
    stagingBuffer
  );
  
  // Copy data to staging buffer
  void* mappedData;
  vkMapMemory(m_device, stagingBuffer.memory, 0, imageSize, 0, &mappedData);
  memcpy(mappedData, data, static_cast<size_t>(imageSize));
  vkUnmapMemory(m_device, stagingBuffer.memory);
  
  stbi_image_free(data);
  
  // Create image and transfer data
  createImage(width, height, VK_FORMAT_R8G8B8A8_UNORM, VK_IMAGE_TILING_OPTIMAL,
             VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT,
             VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, outTexture.image, outTexture.memory);
  
  // Transition, copy, and finalize
  transitionImageLayout(outTexture.image, VK_FORMAT_R8G8B8A8_UNORM, 
                       VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);
  copyBufferToImage(stagingBuffer.buffer, outTexture.image, width, height);
  transitionImageLayout(outTexture.image, VK_FORMAT_R8G8B8A8_UNORM,
                       VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
  
  // Create image view and sampler
  createImageView(outTexture.image, VK_FORMAT_R8G8B8A8_UNORM, outTexture.view);
  createSampler(outTexture.sampler);
  
  outTexture.width = width;
  outTexture.height = height;
  
  // Cleanup staging buffer
  vkDestroyBuffer(m_device, stagingBuffer.buffer, nullptr);
  vkFreeMemory(m_device, stagingBuffer.memory, nullptr);
  
  return true;
}
```

### 6. [ ] Scene Graph Basics 📙
- Create entity-component system
- Implement scene hierarchy
- Set up transform management
- Basic object culling
- **Learning**: [Entity Component Systems](https://austinmorlan.com/posts/entity_component_system/), "Game Engine Architecture" by Jason Gregory
```cpp
class Entity {
public:
  Entity(Scene* scene, uint32_t id) : m_scene(scene), m_id(id) {}
  
  template<typename T, typename... Args>
  T& addComponent(Args&&... args) {
    return m_scene->registry.emplace<T>(m_id, std::forward<Args>(args)...);
  }
  
  template<typename T>
  T& getComponent() {
    return m_scene->registry.get<T>(m_id);
  }
  
  template<typename T>
  bool hasComponent() {
    return m_scene->registry.has<T>(m_id);
  }
  
  template<typename T>
  void removeComponent() {
    m_scene->registry.remove<T>(m_id);
  }
  
  bool isValid() const {
    return m_scene->registry.valid(m_id);
  }
  
  uint32_t getId() const { return m_id; }
  
private:
  Scene* m_scene;
  uint32_t m_id;
};

class Scene {
public:
  Entity createEntity() {
    return Entity(this, registry.create());
  }
  
  void destroyEntity(Entity entity) {
    registry.destroy(entity.getId());
  }
  
  void update(float deltaTime) {
    // Update transformations
    auto transformView = registry.view<TransformComponent, HierarchyComponent>();
    for (auto entity : transformView) {
      auto& transform = transformView.get<TransformComponent>(entity);
      auto& hierarchy = transformView.get<HierarchyComponent>(entity);
      
      if (hierarchy.parent != entt::null) {
        auto& parentTransform = registry.get<TransformComponent>(hierarchy.parent);
        transform.worldMatrix = parentTransform.worldMatrix * transform.localMatrix;
      } else {
        transform.worldMatrix = transform.localMatrix;
      }
    }
    
    // Update systems
    for (auto& system : m_systems) {
      system->update(registry, deltaTime);
    }
  }
  
  entt::registry registry;
  
private:
  std::vector<std::unique_ptr<System>> m_systems;
};
```

### 7. [ ] Input Handling Abstraction 📗
- Create input manager
- Implement device abstraction
- Add input mapping
- Support for keyboard, mouse, and gamepads
- **Learning**: [SDL Input Handling](https://wiki.libsdl.org/SDL3/CategoryInput), Game Programming Patterns (Command pattern)
```cpp
class InputManager {
public:
  enum class InputState {
    PRESSED,
    RELEASED,
    DOWN,
    UP
  };
  
  bool initialize() {
    m_keyboard.fill(false);
    m_keyboardPrevious.fill(false);
    m_mouse.fill(false);
    m_mousePrevious.fill(false);
    
    return true;
  }
  
  void update() {
    // Copy current state to previous
    m_keyboardPrevious = m_keyboard;
    m_mousePrevious = m_mouse;
    
    // Update mouse position
    int x, y;
    uint32_t buttons = SDL_GetMouseState(&x, &y);
    m_mouseX = x;
    m_mouseY = y;
    m_mouse[0] = (buttons & SDL_BUTTON_LMASK) != 0;
    m_mouse[1] = (buttons & SDL_BUTTON_RMASK) != 0;
    m_mouse[2] = (buttons & SDL_BUTTON_MMASK) != 0;
  }
  
  void processEvent(const SDL_Event& event) {
    if (event.type == SDL_EVENT_KEY_DOWN) {
      m_keyboard[event.key.keysym.scancode] = true;
    }
    else if (event.type == SDL_EVENT_KEY_UP) {
      m_keyboard[event.key.keysym.scancode] = false;
    }
  }
  
  bool isKeyDown(SDL_Scancode key) const {
    return m_keyboard[key];
  }
  
  bool isKeyPressed(SDL_Scancode key) const {
    return m_keyboard[key] && !m_keyboardPrevious[key];
  }
  
  bool isKeyReleased(SDL_Scancode key) const {
    return !m_keyboard[key] && m_keyboardPrevious[key];
  }
  
  // Similar methods for mouse and gamepad
  
private:
  std::array<bool, SDL_NUM_SCANCODES> m_keyboard;
  std::array<bool, SDL_NUM_SCANCODES> m_keyboardPrevious;
  std::array<bool, 5> m_mouse;
  std::array<bool, 5> m_mousePrevious;
  int m_mouseX = 0;
  int m_mouseY = 0;
  float m_mouseWheel = 0.0f;
};
```

### 8. [ ] Swapchain Recreation 📙
- Handle window resize events
- Implement swapchain recreation logic
- Update viewport and scissor on resize
- **Learning**: [Vulkan Swapchain Recreation](https://vulkan-tutorial.com/en/Drawing_a_triangle/Swap_chain_recreation)
```cpp
void VulkanRenderer::recreateSwapchain() {
  // Wait for device to be idle
  vkDeviceWaitIdle(m_device);
  
  // Clean up old swapchain resources
  cleanupSwapchain();
  
  // Get new window size
  int width, height;
  SDL_GetWindowSize(m_window, &width, &height);
  
  // Recreate swapchain
  createSwapchain();
  createImageViews();
  createRenderPass();
  createGraphicsPipeline();
  createFramebuffers();
  createCommandBuffers();
  
  // Update descriptor sets if necessary
  updateDescriptorSets();
}

void VulkanRenderer::handleWindowResize(int width, int height) {
  m_framebufferResized = true;
}

void VulkanRenderer::checkRecreateSwapchain() {
  if (m_framebufferResized) {
    recreateSwapchain();
    m_framebufferResized = false;
  }
}
```

### 9. [ ] Texture & Material System 📙
- Create texture manager
- Implement material system
- Add shader uniform handling
- Support for PBR materials
- **Learning**: [Physically Based Rendering](https://learnopengl.com/PBR/Theory), [Vulkan Texturing](https://vkguide.dev/docs/chapter-4/textures/)
```cpp
class Material {
public:
  enum class Type {
    BASIC,
    PBR,
    UNLIT
  };
  
  Material(Type type = Type::BASIC) : m_type(type) {}
  
  void setTexture(const std::string& name, const std::shared_ptr<Texture>& texture) {
    m_textures[name] = texture;
  }
  
  std::shared_ptr<Texture> getTexture(const std::string& name) const {
    auto it = m_textures.find(name);
    if (it != m_textures.end()) {
      return it->second;
    }
    return nullptr;
  }
  
  template<typename T>
  void setParameter(const std::string& name, const T& value) {
    // Store type-erased value
    m_parameters[name] = value;
  }
  
  template<typename T>
  T getParameter(const std::string& name, const T& defaultValue) const {
    auto it = m_parameters.find(name);
    if (it != m_parameters.end()) {
      return std::any_cast<T>(it->second);
    }
    return defaultValue;
  }
  
  void bindDescriptorSet(VkCommandBuffer cmdBuffer, VkPipelineLayout layout, uint32_t index) {
    vkCmdBindDescriptorSets(cmdBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS,
                           layout, 0, 1, &m_descriptorSet, 0, nullptr);
  }
  
private:
  Type m_type;
  std::unordered_map<std::string, std::shared_ptr<Texture>> m_textures;
  std::unordered_map<std::string, std::any> m_parameters;
  VkDescriptorSet m_descriptorSet = VK_NULL_HANDLE;
};
```

### 10. [ ] Basic Lighting Pass 📕
- Implement deferred rendering
- Create G-buffer
- Add basic lighting models
- Support for multiple light types
- **Learning**: [Deferred Shading](https://learnopengl.com/Advanced-Lighting/Deferred-Shading), "Real-Time Rendering" by Tomas Akenine-Möller
```cpp
void DeferredRenderer::render(const Scene& scene, const Camera& camera) {
  // Geometry pass
  beginRenderPass(m_gBufferPass);
  
  // Bind G-buffer pipeline
  vkCmdBindPipeline(m_commandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, m_gBufferPipeline);
  
  // Set viewport and scissor
  VkViewport viewport{};
  viewport.width = static_cast<float>(m_width);
  viewport.height = static_cast<float>(m_height);
  viewport.maxDepth = 1.0f;
  vkCmdSetViewport(m_commandBuffer, 0, 1, &viewport);
  
  VkRect2D scissor{};
  scissor.extent = {m_width, m_height};
  vkCmdSetScissor(m_commandBuffer, 0, 1, &scissor);
  
  // Push camera constants
  PushConstantBlock pushConstants{};
  pushConstants.view = camera.getViewMatrix();
  pushConstants.projection = camera.getProjectionMatrix();
  
  vkCmdPushConstants(m_commandBuffer, m_pipelineLayout, VK_SHADER_STAGE_VERTEX_BIT,
                    0, sizeof(PushConstantBlock), &pushConstants);
  
  // Render all objects
  auto view = scene.registry.view<MeshComponent, TransformComponent, MaterialComponent>();
  for (auto entity : view) {
    auto& mesh = view.get<MeshComponent>(entity);
    auto& transform = view.get<TransformComponent>(entity);
    auto& material = view.get<MaterialComponent>(entity);
    
    // Update model matrix
    pushConstants.model = transform.worldMatrix;
    vkCmdPushConstants(m_commandBuffer, m_pipelineLayout, VK_SHADER_STAGE_VERTEX_BIT,
                      0, sizeof(PushConstantBlock), &pushConstants);
    
    // Bind material
    material.material->bindDescriptorSet(m_commandBuffer, m_pipelineLayout, 0);
    
    // Bind mesh and draw
    mesh.mesh->bind(m_commandBuffer);
    mesh.mesh->draw(m_commandBuffer);
  }
  
  endRenderPass();
  
  // Lighting pass
  beginRenderPass(m_lightingPass);
  
  // Bind lighting pipeline
  vkCmdBindPipeline(m_commandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, m_lightingPipeline);
  
  // Set viewport and scissor
  vkCmdSetViewport(m_commandBuffer, 0, 1, &viewport);
  vkCmdSetScissor(m_commandBuffer, 0, 1, &scissor);
  
  // Bind G-buffer textures
  vkCmdBindDescriptorSets(m_commandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS,
                         m_lightingPipelineLayout, 0, 1, &m_gBufferDescriptorSet, 0, nullptr);
  
  // Draw lights
  LightingPushConstants lightConstants{};
  lightConstants.viewPos = glm::vec4(camera.getPosition(), 1.0f);
  lightConstants.numLights = 0;
  
  // Collect lights
  auto lightView = scene.registry.view<LightComponent, TransformComponent>();
  for (auto entity : lightView) {
    auto& light = lightView.get<LightComponent>(entity);
    auto& transform = lightView.get<TransformComponent>(entity);
    
    if (lightConstants.numLights < MAX_LIGHTS) {
      lightConstants.lights[lightConstants.numLights].position = 
        glm::vec4(transform.worldMatrix[3]) * glm::vec4(1.0f, 1.0f, 1.0f, 0.0f);
      lightConstants.lights[lightConstants.numLights].color = light.color;
      lightConstants.lights[lightConstants.numLights].radius = light.radius;
      lightConstants.lights[lightConstants.numLights].intensity = light.intensity;
      lightConstants.numLights++;
    }
  }
  
  vkCmdPushConstants(m_commandBuffer, m_lightingPipelineLayout,
                    VK_SHADER_STAGE_FRAGMENT_BIT, 0,
                    sizeof(LightingPushConstants), &lightConstants);
  
  // Draw fullscreen quad
  vkCmdDraw(m_commandBuffer, 6, 1, 0, 0);
  
  endRenderPass();
}
```

### 11. [ ] Model Loading and Camera 📙
- Integrate assimp for model loading
- Create model/mesh hierarchy
- Implement camera controller
- Add frustum culling
- **Learning**: [Assimp Model Loading](http://assimp.sourceforge.net/lib_html/index.html), "3D Game Engine Design" by David H. Eberly
```cpp
bool ModelLoader::loadModel(const std::string& filename, Model& outModel) {
  Assimp::Importer importer;
  const aiScene* scene = importer.ReadFile(
    filename,
    aiProcess_Triangulate |
    aiProcess_GenSmoothNormals |
    aiProcess_FlipUVs |
    aiProcess_CalcTangentSpace
  );
  
  if (!scene || scene->mFlags & AI_SCENE_FLAGS_INCOMPLETE || !scene->mRootNode) {
    m_logger.error("Failed to load model: {}", importer.GetErrorString());
    return false;
  }
  
  std::string directory = filename.substr(0, filename.find_last_of('/'));
  
  // Process materials
  for (unsigned int i = 0; i < scene->mNumMaterials; i++) {
    aiMaterial* material = scene->mMaterials[i];
    std::shared_ptr<Material> newMaterial = processMaterial(material, directory);
    outModel.materials.push_back(newMaterial);
  }
  
  // Process meshes
  processNode(scene->mRootNode, scene, outModel);
  
  return true;
}

void ModelLoader::processNode(aiNode* node, const aiScene* scene, Model& outModel) {
  // Process meshes in current node
  for (unsigned int i = 0; i < node->mNumMeshes; i++) {
    aiMesh* mesh = scene->mMeshes[node->mMeshes[i]];
    outModel.meshes.push_back(processMesh(mesh, scene, outModel));
  }
  
  // Recursively process child nodes
  for (unsigned int i = 0; i < node->mNumChildren; i++) {
    processNode(node->mChildren[i], scene, outModel);
  }
}

std::shared_ptr<Mesh> ModelLoader::processMesh(aiMesh* mesh, const aiScene* scene, Model& model) {
  std::vector<Vertex> vertices;
  std::vector<uint32_t> indices;
  
  // Process vertices
  for (unsigned int i = 0; i < mesh->mNumVertices; i++) {
    Vertex vertex{};
    
    // Position
    vertex.position.x = mesh->mVertices[i].x;
    vertex.position.y = mesh->mVertices[i].y;
    vertex.position.z = mesh->mVertices[i].z;
    
    // Normal
    if (mesh->HasNormals()) {
      vertex.normal.x = mesh->mNormals[i].x;
      vertex.normal.y = mesh->mNormals[i].y;
      vertex.normal.z = mesh->mNormals[i].z;
    }
    
    // Texture coordinates
    if (mesh->mTextureCoords[0]) {
      vertex.texCoord.x = mesh->mTextureCoords[0][i].x;
      vertex.texCoord.y = mesh->mTextureCoords[0][i].y;
    } else {
      vertex.texCoord = glm::vec2(0.0f, 0.0f);
    }
    
    // Tangent
    if (mesh->HasTangentsAndBitangents()) {
      vertex.tangent.x = mesh->mTangents[i].x;
      vertex.tangent.y = mesh->mTangents[i].y;
      vertex.tangent.z = mesh->mTangents[i].z;
    }
    
    vertices.push_back(vertex);
  }
  
  // Process indices
  for (unsigned int i = 0; i < mesh->mNumFaces; i++) {
    aiFace face = mesh->mFaces[i];
    for (unsigned int j = 0; j < face.mNumIndices; j++) {
      indices.push_back(face.mIndices[j]);
    }
  }
  
  // Create Vulkan mesh
  auto newMesh = std::make_shared<Mesh>();
  newMesh->createFromVertices(m_device, vertices, indices, mesh->mMaterialIndex);
  
  return newMesh;
}
```

### 12. [ ] Refinement and Optimization 📕
- Implement multithreading
- Add memory pooling
- Optimize render batches
- Implement frustum culling
- Add level of detail system
- **Learning**: [Vulkan Memory Management](https://gpuopen.com/vulkan-memory-management/), "Efficient C++" by Dov Bulka
```cpp
class RenderQueue {
public:
  struct RenderCommand {
    std::shared_ptr<Mesh> mesh;
    std::shared_ptr<Material> material;
    glm::mat4 transform;
    float distance; // For sorting
  };
  
  void addCommand(const RenderCommand& command) {
    m_commands.push_back(command);
  }
  
  void sort() {
    // Sort back-to-front for transparent objects
    std::sort(m_transparentCommands.begin(), m_transparentCommands.end(),
             [](const RenderCommand& a, const RenderCommand& b) {
               return a.distance > b.distance;
             });
    
    // Sort front-to-back for opaque objects (better z-culling)
    std::sort(m_opaqueCommands.begin(), m_opaqueCommands.end(),
             [](const RenderCommand& a, const RenderCommand& b) {
               return a.distance < b.distance;
             });
    
    // Sort by material to minimize state changes
    std::sort(m_opaqueCommands.begin(), m_opaqueCommands.end(),
             [](const RenderCommand& a, const RenderCommand& b) {
               return a.material->getId() < b.material->getId();
             });
  }
  
  void clear() {
    m_opaqueCommands.clear();
    m_transparentCommands.clear();
  }
  
  const std::vector<RenderCommand>& getOpaqueCommands() const { return m_opaqueCommands; }
  const std::vector<RenderCommand>& getTransparentCommands() const { return m_transparentCommands; }
  
private:
  std::vector<RenderCommand> m_opaqueCommands;
  std::vector<RenderCommand> m_transparentCommands;
};
```

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