cmake_minimum_required(VERSION 3.20)

# Include CTest for test discovery
include(CTest)

# Fetch Catch2 for testing
include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.8.1
)
FetchContent_MakeAvailable(Catch2)

# Add the test executables
add_executable(core_tests
    core_tests.cpp
)
target_link_libraries(core_tests
    PRIVATE
    praxis_core
    Catch2::Catch2WithMain
)

add_executable(graphics_tests
    graphics_tests.cpp
)
target_link_libraries(graphics_tests
    PRIVATE
    praxis_core
    Catch2::Catch2WithMain
)


# Register tests with CTest
add_test(NAME CoreTests COMMAND core_tests)
add_test(NAME GraphicsTests COMMAND graphics_tests)
