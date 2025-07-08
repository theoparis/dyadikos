cmake_minimum_required(VERSION 4.0)

project(dyadikos LANGUAGES C CXX ASM)

set(BUILD_SHARED_LIBS OFF CACHE BOOL "Force shared libraries" FORCE)
set(SDL_SHARED OFF CACHE BOOL "" FORCE)
set(SDL_STATIC ON CACHE BOOL "" FORCE)

set(SDL_RENDER OFF CACHE BOOL "" FORCE)
set(SDL_GPU OFF CACHE BOOL "" FORCE)
set(SDL_JOYSTICK OFF CACHE BOOL "" FORCE)
set(SDL_HAPTIC OFF CACHE BOOL "" FORCE)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE TRUE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(CMAKE_C_VISIBILITY_PRESET hidden)
set(CMAKE_C_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_CXX_VISIBILITY_INLINES_HIDDEN ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_CLANG_TIDY clang-tidy)

# download CPM.cmake
file(
    DOWNLOAD
    https://github.com/cpm-cmake/CPM.cmake/releases/download/v0.42.0/CPM.cmake
    ${CMAKE_CURRENT_BINARY_DIR}/cmake/CPM.cmake
)
include(${CMAKE_CURRENT_BINARY_DIR}/cmake/CPM.cmake)

CPMAddPackage("gh:KhronosGroup/Vulkan-Headers@1.4.321")
CPMAddPackage("gh:g-truc/glm#2d4c4b4")
CPMAddPackage("gh:libsdl-org/SDL#6386781")

add_library(
    dyadikos
    src/app.cpp
    src/transform.cpp
)
target_include_directories(
    dyadikos
    PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)
target_link_libraries(dyadikos PUBLIC Vulkan-Headers glm::glm SDL3::SDL3)
target_compile_features(dyadikos PUBLIC cxx_std_26)
target_compile_definitions(dyadikos PUBLIC GLM_ENABLE_EXPERIMENTAL)

add_executable(dyadikos-demo examples/rainbow_window.cpp)
target_link_libraries(dyadikos-demo PRIVATE dyadikos)
target_compile_features(dyadikos-demo PRIVATE cxx_std_26)
