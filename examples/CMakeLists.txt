cmake_minimum_required(VERSION 3.20)

# Basic application example
add_executable(basic_app
    basic_app.cpp
)

target_link_libraries(basic_app
    PRIVATE
    praxis_core
)

# Install the example applications
install(TARGETS basic_app
    RUNTIME DESTINATION bin/examples
) 