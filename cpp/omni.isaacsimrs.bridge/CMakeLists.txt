cmake_minimum_required(VERSION 3.20)
project(omni_isaacsimrs_bridge LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(NOT DEFINED ENV{ISAAC_SIM_PATH})
    message(FATAL_ERROR "ISAAC_SIM_PATH env var must point to the Isaac Sim install root (the directory containing kit/dev/include)")
endif()
set(ISAAC_SIM "$ENV{ISAAC_SIM_PATH}")

add_library(omni_isaacsimrs_bridge SHARED
    plugins/PluginInterface.cpp
)

set_target_properties(omni_isaacsimrs_bridge PROPERTIES
    OUTPUT_NAME "omni.isaacsimrs.bridge.plugin"
    PREFIX "lib"
    SUFFIX ".so"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/bin"
)

target_include_directories(omni_isaacsimrs_bridge PRIVATE
    "${ISAAC_SIM}/kit/dev/include"
)

target_link_libraries(omni_isaacsimrs_bridge PRIVATE dl)
