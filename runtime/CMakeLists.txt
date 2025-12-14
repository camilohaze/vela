cmake_minimum_required(VERSION 3.10)
project(vela_runtime C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Enable warnings
if(MSVC)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Include directories
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

# Source files
set(RUNTIME_SOURCES
    src/gc.c
    src/signals.c
    src/actors.c
    src/runtime.c
)

# Create static library
add_library(vela_runtime_static STATIC ${RUNTIME_SOURCES})
set_target_properties(vela_runtime_static PROPERTIES
    OUTPUT_NAME vela_runtime
    ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
)

# Create shared library
add_library(vela_runtime_shared SHARED ${RUNTIME_SOURCES})
set_target_properties(vela_runtime_shared PROPERTIES
    OUTPUT_NAME vela_runtime
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin
)

# Find required libraries
find_package(Threads REQUIRED)

# Link libraries
if(WIN32)
    target_link_libraries(vela_runtime_static Threads::Threads)
    target_link_libraries(vela_runtime_shared Threads::Threads)
else()
    target_link_libraries(vela_runtime_static Threads::Threads m)
    target_link_libraries(vela_runtime_shared Threads::Threads m)
endif()

# Installation
install(TARGETS vela_runtime_static vela_runtime_shared
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    RUNTIME DESTINATION bin
)

install(DIRECTORY include/
    DESTINATION include/vela
    FILES_MATCHING PATTERN "*.h"
)

# Build examples/tests (optional)
option(BUILD_EXAMPLES "Build example programs" OFF)
if(BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

# Build tests (optional)
option(BUILD_TESTS "Build test programs" OFF)
if(BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# Generate pkg-config file
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/vela_runtime.pc.in
    ${CMAKE_BINARY_DIR}/vela_runtime.pc
    @ONLY
)

install(FILES ${CMAKE_BINARY_DIR}/vela_runtime.pc
    DESTINATION lib/pkgconfig
)