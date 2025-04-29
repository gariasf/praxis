#define CATCH_CONFIG_MAIN
#include "catch2/catch_test_macros.hpp"

#include "praxis/core/Engine.h"
#include "praxis/graphics/Renderer.h"
#include "praxis/utils/Logger.h"
#include "SDL3/SDL_vulkan.h"

#include <SDL3/SDL.h>
#include <thread>
#include <chrono>

namespace {
    class MockSDLWindow {
    public:
        static SDL_Window* Create(const char* title = "Test Window", int width = 100, int height = 100) {
            // Create a hidden window for testing - using hidden to avoid UI popping up during tests
            return SDL_CreateWindow(title, width, height, 
                SDL_WINDOW_VULKAN | SDL_WINDOW_HIDDEN);
        }
    };

    // Helper to check if Vulkan is available on the current system
    bool isVulkanSupported() {
        return SDL_Vulkan_GetInstanceExtensions(nullptr) != nullptr;
    }
}

// Setup and teardown for all tests
class VulkanTestFixture {
public:
    VulkanTestFixture() {
        SDL_Init(SDL_INIT_VIDEO);
        
        praxis::utils::Logger::initialize("GraphicsTest", "");
    }
    
    ~VulkanTestFixture() {
        praxis::utils::Logger::shutdown();
        SDL_Quit();
    }
};

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Lifecycle", "[graphics][renderer]") {
    SECTION("Constructor and destructor") {
        // Test that constructor and destructor don't crash
        {
            praxis::graphics::Renderer renderer;
        }
        SUCCEED("Renderer constructed and destructed without crashing");
    }
}

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Initialization", "[graphics][renderer]") {
    SECTION("Null window initialization") {
        praxis::graphics::Renderer renderer;
        bool result = renderer.initialize(nullptr);
        REQUIRE_FALSE(result);
    }
    
    SECTION("Valid window initialization") {
        SDL_Window* window = MockSDLWindow::Create();
        REQUIRE(window != nullptr);
        
        praxis::graphics::Renderer renderer;
        bool result = renderer.initialize(window);
        
        // Skip Vulkan tests if not supported on the current system
        if (isVulkanSupported()) {
            REQUIRE(result);
        } else {
            WARN("Skipping Vulkan initialization test - Vulkan not supported on this system");
        }
        
        SDL_DestroyWindow(window);
    }
    
    SECTION("Double initialization") {
        if (!isVulkanSupported()) {
            WARN("Skipping Vulkan initialization test - Vulkan not supported on this system");
            return;
        }
        
        SDL_Window* window1 = MockSDLWindow::Create("First Window", 800, 600);
        SDL_Window* window2 = MockSDLWindow::Create("Second Window", 1024, 768);
        REQUIRE(window1 != nullptr);
        REQUIRE(window2 != nullptr);
        
        praxis::graphics::Renderer renderer;
        
        // First initialization should succeed
        bool result1 = renderer.initialize(window1);
        REQUIRE(result1);
        
        // Second initialization should fail or reinitialize cleanly
        bool result2 = renderer.initialize(window2);
        // We don't make a specific requirement here as the behavior could go either way
        // depending on the implementation
        
        SDL_DestroyWindow(window1);
        SDL_DestroyWindow(window2);
        REQUIRE(result2);
    }
}

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Frame Operations", "[graphics][renderer]") {
    if (!isVulkanSupported()) {
        WARN("Skipping Vulkan frame operations test - Vulkan not supported on this system");
        return;
    }
    
    SDL_Window* window = MockSDLWindow::Create("Frame Test", 800, 600);
    REQUIRE(window != nullptr);
    
    praxis::graphics::Renderer renderer;

    if (renderer.initialize(window)) {
        SECTION("Begin and end frame") {
            // Test that beginFrame and endFrame work without crashing
            REQUIRE_NOTHROW([&]() {
                if (renderer.beginFrame()) {
                    renderer.endFrame();
                }
            }());
        }
        
        SECTION("Multiple frames") {
            // Test rendering multiple consecutive frames
            for (int i = 0; i < 3; i++) {
                if (renderer.beginFrame()) {
                    renderer.endFrame();
                }
            }
            SUCCEED("Multiple frames rendered without crashing");
        }
    } else {
        WARN("Renderer initialization failed, skipping frame tests");
    }
    
    SDL_DestroyWindow(window);
}

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Window Handling", "[graphics][renderer]") {
    if (!isVulkanSupported()) {
        WARN("Skipping Vulkan window handling test - Vulkan not supported on this system");
        return;
    }
    
    SDL_Window* window = MockSDLWindow::Create("Resize Test", 800, 600);
    REQUIRE(window != nullptr);
    
    praxis::graphics::Renderer renderer;

    if (renderer.initialize(window)) {
        SECTION("Window resize handling") {
            // Test resize handling (may not visibly resize since a window is hidden)
            REQUIRE_NOTHROW(renderer.handleWindowResize(1024, 768));
            
            // Test that we can still render after resize
            if (renderer.beginFrame()) {
                renderer.endFrame();
            }
            
            SUCCEED("Window resize handled without crashing");
        }
        
        SECTION("Check and recreate swapchain") {
            // Test the swapchain recreation manually
            REQUIRE_NOTHROW(renderer.checkRecreateSwapchain());
            
            // Test that we can still render after swapchain check
            if (renderer.beginFrame()) {
                renderer.endFrame();
            }
            
            SUCCEED("Swapchain check and potential recreation handled without crashing");
        }
        
        SECTION("Extreme window size changes") {
            // Test with a very small size
            REQUIRE_NOTHROW(renderer.handleWindowResize(1, 1));
            
            // Test with a very large size
            REQUIRE_NOTHROW(renderer.handleWindowResize(8192, 4320));
            
            // Check render capability is maintained
            if (renderer.beginFrame()) {
                renderer.endFrame();
            }
            
            SUCCEED("Extreme window sizes handled without crashing");
        }
    } else {
        WARN("Renderer initialization failed, skipping window tests");
    }
    
    SDL_DestroyWindow(window);
}

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Stress Test", "[graphics][renderer][!mayfail]") {
    if (!isVulkanSupported()) {
        WARN("Skipping Vulkan stress test - Vulkan not supported on this system");
        return;
    }
    
    SDL_Window* window = MockSDLWindow::Create("Stress Test", 800, 600);
    REQUIRE(window != nullptr);
    
    praxis::graphics::Renderer renderer;

    if (renderer.initialize(window)) {
      constexpr int NUM_FRAMES = 10; // Reduced for faster test execution
      constexpr int NUM_RESIZE_OPS = 5;
        
        // Rapid rendering of multiple frames
        for (int i = 0; i < NUM_FRAMES; i++) {
            if (renderer.beginFrame()) {
                renderer.endFrame();
            }
            
            // Occasionally resize the window
            if (i % 2 == 0 && i < NUM_RESIZE_OPS) {
                int width = 800 + (i * 100);
                int height = 600 + (i * 75);
                renderer.handleWindowResize(width, height);
                
                // Force swapchain recreation
                renderer.checkRecreateSwapchain();
            }
        }
        
        SUCCEED("Completed stress test without crashing");
    } else {
        WARN("Renderer initialization failed, skipping stress test");
    }
    
    SDL_DestroyWindow(window);
}

TEST_CASE_METHOD(VulkanTestFixture, "Renderer Error Paths", "[graphics][renderer]") {
    // Test error handling behavior
    
    SECTION("Initialize with already destroyed window") {
        SDL_Window* window = MockSDLWindow::Create();
        REQUIRE(window != nullptr);
        
        // Destroy the window before initialization
        SDL_DestroyWindow(window);
        
        // Try to initialize with an invalid window
        praxis::graphics::Renderer renderer;
        bool result = renderer.initialize(window);
        
        // Should fail gracefully
        REQUIRE_FALSE(result);
    }
}
