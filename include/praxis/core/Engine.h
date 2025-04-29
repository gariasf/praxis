#pragma once

#include <memory>
#include <string>
#include <vector>

// Forward declarations
struct SDL_Window;

namespace praxis {

// Forward declarations
namespace graphics {
  class Renderer;
}

namespace core {

  /**
   * @class Engine
   * @brief Main engine class responsible for initialization, main loop and cleanup
   */
  class Engine {
  public:
    /**
     * @brief Default constructor
     */
    Engine();

    /**
     * @brief Destructor
     */
    ~Engine();

    /**
     * @brief Initializes the engine subsystems
     * @param appName The name of the application
     * @param width Initial window width
     * @param height Initial window height
     * @return True if initialization succeeded, false otherwise
     */
    bool initialize(const std::string& appName, int width = 1280, int height = 720);

    /**
     * @brief Runs the main engine loop
     * @return Exit code
     */
    int run();

    /**
     * @brief Shuts down all engine subsystems
     */
    void shutdown();

    /**
     * @brief Signals the engine to stop running
     */
    void stop();

    /**
     * @brief Get the SDL window
     * @return Pointer to the SDL window
     */
    SDL_Window* getWindow() const { return m_window; }

  private:
    /**
     * @brief Processes SDL events
     */
    void processEvents();

    /**
     * @brief Updates all engine systems
     * @param deltaTime Time elapsed since last update in seconds
     */
    static void update(float deltaTime);

    /**
     * @brief Renders the current frame
     */
    void render() const;

    /**
     * @brief Calculates the delta time between frames
     * @return Delta time in seconds
     */
    float calculateDeltaTime();

  private:
    bool m_running;
    bool m_initialized;
    SDL_Window* m_window;
    uint64_t m_lastFrameTime;
    std::unique_ptr<graphics::Renderer> m_renderer;
    std::string m_appName;
  };

} // namespace core
} // namespace praxis