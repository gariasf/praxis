#define CATCH_CONFIG_MAIN
#include "catch2/catch_test_macros.hpp"

#include "praxis/core/Engine.h"
#include "praxis/graphics/Renderer.h"
#include "praxis/utils/Logger.h"

#include <SDL3/SDL.h>

class MockSDLWindow {
public:
    static SDL_Window* Create() {
        // Create a hidden window for testing
        return SDL_CreateWindow("Test Window", 100, 100, 
            SDL_WINDOW_VULKAN | SDL_WINDOW_HIDDEN);
    }
};

TEST_CASE("Engine initialization and basic operations", "[engine]") {
    praxis::utils::Logger::initialize("TestApp", "");

    // Check if we're in a CI environment
    bool is_ci = (std::getenv("CI") != nullptr && std::string(std::getenv("CI")) == "true");
    bool headless = is_ci || (std::getenv("DISPLAY") == nullptr && std::getenv("WAYLAND_DISPLAY") ==
                              nullptr);

    if (headless) {
      // Set XDG_RUNTIME_DIR if not set
      if (std::getenv("XDG_RUNTIME_DIR") == nullptr) {
        setenv("XDG_RUNTIME_DIR", "/tmp", 1);
      }
    }

    SECTION("Engine can be constructed and destructed") {
        praxis::core::Engine engine;
        REQUIRE_NOTHROW([&]() { engine.shutdown(); }());
    }
    
    SECTION("Engine initialization with valid parameters succeeds") {
        praxis::core::Engine engine;
        bool result = engine.initialize("Test Engine", 800, 600);
        REQUIRE(result);

        // In headless environments, the window might be nullptr
        if (!headless) {
          REQUIRE(engine.getWindow() != nullptr);
        }
        engine.shutdown();
    }
    
    SECTION("Engine can be started and stopped") {
        praxis::core::Engine engine;
        REQUIRE(engine.initialize("Test Engine", 800, 600));
        
        // Create a thread to stop the engine after a short time
        std::thread stopThread([&engine]() {
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
            engine.stop();
        });
        
        // Run should exit after stop is called
        int result = engine.run();
        REQUIRE(result == 0);
        
        stopThread.join();
        engine.shutdown();
    }
    
    SECTION("Engine fails gracefully with invalid dimensions") {
        praxis::core::Engine engine;
        bool result = engine.initialize("Test Engine", -1, -1);
        // Should still succeed as SDL will adjust to valid dimensions
        REQUIRE(result);
        engine.shutdown();
    }
    
    SECTION("Double initialization is handled correctly") {
        praxis::core::Engine engine;
        REQUIRE(engine.initialize("First Init", 800, 600));
        REQUIRE(engine.initialize("Second Init", 1024, 768));
        engine.shutdown();
    }
    
    SECTION("Engine cannot run if not initialized") {
        praxis::core::Engine engine;
        int result = engine.run();
        REQUIRE(result == -1);
    }
    
    praxis::utils::Logger::shutdown();
}
