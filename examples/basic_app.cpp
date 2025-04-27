#include "praxis/core/Engine.h"
#include "praxis/utils/Logger.h"

int main(int argc, char* argv[]) {
    // Initialize the engine
    praxis::core::Engine engine;
    
    // Initialize with application name and window size
    if (!engine.initialize("Praxis Basic Example", 1280, 720)) {
        return -1;
    }
    
    // Run the main loop
    int result = engine.run();
    
    // Engine will be automatically cleaned up when it goes out of scope
    
    return result;
} 