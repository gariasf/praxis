#include "praxis/core/Engine.h"

#include "praxis/graphics/Renderer.h"
#include "praxis/utils/Logger.h"

#include <SDL3/SDL.h>
#include <chrono>
#include <iostream>

/**
 * TODO: possible improvements:
 * - Use `std::chrono` for time measurement instead of SDL_GetTicks
 * - Hide SDL specific code behind a platform abstraction layer eventually
 * - Inject logger (DI) instead of accessing Logger as a static singleton everywhere 
 * - How to control or make sure that `m_window` is properly freed in case of exceptions before we
 *   shutdown?
 */

namespace praxis::core {

Engine::Engine()
    : m_running(false), m_initialized(false), m_window(nullptr), m_lastFrameTime(0),
      m_renderer(nullptr), m_appName("Praxis Engine") {}

Engine::~Engine() { shutdown(); }

// TODO: We're not handling exceptions.
bool Engine::initialize(const std::string& appName, int width, int height) {
  if (m_initialized) {
    utils::Logger::warn("Engine already initialized");
    return true;
  }

  m_appName = appName;

  // Initialize logger
  if (!utils::Logger::initialize(appName, appName + ".log")) {
    std::cerr << "Failed to initialize logger" << std::endl;
    return false;
  }

  utils::Logger::info("Initializing Praxis Engine");

  // Initialize SDL
  if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) < 0) {
    utils::Logger::error("SDL initialization failed: {}", SDL_GetError());
    return false;
  }

  // Create a window
  m_window =
      SDL_CreateWindow(m_appName.c_str(), width, height, SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE);

  if (!m_window) {
    utils::Logger::error("Window creation failed: {}", SDL_GetError());
    return false;
  }

  // Create renderer
  m_renderer = std::make_unique<graphics::Renderer>();
  if (!m_renderer->initialize(m_window)) {
    utils::Logger::error("Renderer initialization failed");
    return false;
  }

  m_initialized = true;
  utils::Logger::info("Engine initialized successfully");

  return true;
}

// TODO: We're not handling exceptions.
int Engine::run() {
  if (!m_initialized) {
    utils::Logger::error("Cannot run engine: not initialized");
    return -1;
  }

  utils::Logger::info("Starting engine main loop");

  m_running = true;
  m_lastFrameTime = SDL_GetTicks();

  while (m_running) {
    processEvents();

    float deltaTime = calculateDeltaTime();
    update(deltaTime);
    render();
  }

  utils::Logger::info("Engine main loop ended");

  return 0;
}

void Engine::shutdown() {
  if (!m_initialized) {
    return;
  }

  utils::Logger::info("Shutting down engine");

  if (m_renderer) {
    m_renderer->cleanup();
    m_renderer.reset();
  }

  if (m_window) {
    SDL_DestroyWindow(m_window);
    m_window = nullptr;
  }

  SDL_Quit();

  utils::Logger::info("Engine shutdown complete");
  utils::Logger::shutdown();

  m_initialized = false;
}

void Engine::stop() { m_running = false; }

void Engine::processEvents() {
  SDL_Event event;
  while (SDL_PollEvent(&event)) {
    switch (event.type) {
    case SDL_EVENT_QUIT:
      stop();
      break;
    case SDL_EVENT_WINDOW_RESIZED:
      if (m_renderer) {
        m_renderer->handleWindowResize(event.window.data1, event.window.data2);
      }
      break;
    default:
      break;
    }
  }
}

void Engine::update(float deltaTime) {
  // TODO: Implement engine systems update
}

void Engine::render() const {
  if (!m_renderer) {
    return;
  }

  m_renderer->checkRecreateSwapchain();

  if (!m_renderer->beginFrame()) {
    return;
  }

  // TODO: Implement scene rendering

  m_renderer->endFrame();
}

float Engine::calculateDeltaTime() {
  const uint64_t currentTime = SDL_GetTicks();
  const float deltaTime = (currentTime - m_lastFrameTime) / 1000.0f; // Convert to seconds
  m_lastFrameTime = currentTime;

  return deltaTime;
}

} // namespace praxis::core