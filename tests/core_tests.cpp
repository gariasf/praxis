#include <catch2/catch_test_macros.hpp>

#include "praxis/core/Engine.h"
#include "praxis/utils/Logger.h"

TEST_CASE("Logger initialization", "[utils]") {
    REQUIRE(praxis::utils::Logger::initialize("TestLogger"));
    
    // Log some test messages at different levels
    praxis::utils::Logger::trace("This is a trace message");
    praxis::utils::Logger::debug("This is a debug message");
    praxis::utils::Logger::info("This is an info message");
    praxis::utils::Logger::warn("This is a warning message");
    praxis::utils::Logger::error("This is an error message");
    
    praxis::utils::Logger::shutdown();
}

TEST_CASE("Engine lifecycle", "[core]") {
    praxis::core::Engine engine;
    
    // Initialize with a test name
    bool result = engine.initialize("TestEngine", 800, 600);
    
    // On automated test environments, initialization might fail if no graphics device
    // is available, so we don't strictly REQUIRE it to succeed
    if (result) {
        REQUIRE(engine.getWindow() != nullptr);
        
        // Call stop immediately since we're just testing initialization
        engine.stop();
        
        // We don't test run() since it contains the main loop
        // which would block the test indefinitely
        
        // The engine will be cleaned up automatically when it goes out of scope
    } else {
        WARN("Engine initialization failed. This may be normal in a headless test environment.");
    }
} 