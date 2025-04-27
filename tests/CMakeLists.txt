cmake_minimum_required(VERSION 3.20)

# Include CTest for test discovery
include(CTest)

# Fetch Catch2 for testing
include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.4.0
)
FetchContent_MakeAvailable(Catch2)

# Add the test executable
add_executable(core_tests
    core_tests.cpp
)

target_link_libraries(core_tests
    PRIVATE
    praxis_core
    Catch2::Catch2WithMain
)

# Register tests with CTest
add_test(NAME CoreTests COMMAND core_tests)

# Create a custom target to run all tests
add_custom_target(run_tests ALL
    COMMAND ${CMAKE_CTEST_COMMAND} --output-on-failure
    DEPENDS core_tests
    COMMENT "Running tests..."
) 