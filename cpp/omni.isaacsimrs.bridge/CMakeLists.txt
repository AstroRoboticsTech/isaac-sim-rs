cmake_minimum_required(VERSION 3.20)
project(omni_isaacsimrs_bridge LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(NOT DEFINED ENV{ISAAC_SIM_PATH})
    message(FATAL_ERROR "ISAAC_SIM_PATH env var must point to the Isaac Sim install root (the directory containing kit/dev/include)")
endif()
set(ISAAC_SIM "$ENV{ISAAC_SIM_PATH}")

set(CARGO_TARGET_DIR "$ENV{CARGO_TARGET_DIR}" CACHE PATH "Cargo target directory")
if(NOT CARGO_TARGET_DIR)
    set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target" CACHE PATH "Cargo target directory" FORCE)
endif()
set(CARGO_PROFILE "debug" CACHE STRING "Cargo build profile (debug or release)")
set(RUST_CDYLIB_PATH "${CARGO_TARGET_DIR}/${CARGO_PROFILE}/libisaac_sim_bridge.so")
if(NOT EXISTS "${RUST_CDYLIB_PATH}")
    message(FATAL_ERROR "Rust cdylib not found at ${RUST_CDYLIB_PATH}; run 'cargo build -p isaac-sim-bridge' first")
endif()

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(Ogn)
include(Packman)

packman_pull_usd(USD_ROOT)

file(GLOB OGN_FILES CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/nodes/*.ogn")
set(OGN_GENERATED_HEADERS "")
foreach(ogn_file ${OGN_FILES})
    add_ogn_node(
        OGN_FILE "${ogn_file}"
        EXTENSION "omni.isaacsimrs.bridge"
    )
    get_filename_component(node_name ${ogn_file} NAME_WE)
    list(APPEND OGN_GENERATED_HEADERS "${${node_name}_DATABASE_H}")
endforeach()

file(GLOB_RECURSE PLUGIN_SOURCES CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/plugins/*.cpp")
file(GLOB_RECURSE NODE_SOURCES CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/nodes/*.cpp")
file(GLOB_RECURSE CXX_BRIDGE_SHIMS CONFIGURE_DEPENDS "${CARGO_TARGET_DIR}/cxxbridge/*/src/*.cc")

add_library(omni_isaacsimrs_bridge SHARED
    ${PLUGIN_SOURCES}
    ${NODE_SOURCES}
    ${OGN_GENERATED_HEADERS}
    ${CXX_BRIDGE_SHIMS}
)

set_target_properties(omni_isaacsimrs_bridge PROPERTIES
    OUTPUT_NAME "omni.isaacsimrs.bridge.plugin"
    PREFIX "lib"
    SUFFIX ".so"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/bin"
    BUILD_RPATH "$ORIGIN"
    INSTALL_RPATH "$ORIGIN"
)

target_include_directories(omni_isaacsimrs_bridge PRIVATE
    "${ISAAC_SIM}/kit/dev/include"
    "${ISAAC_SIM}/kit/dev/fabric/include"
    "${ISAAC_SIM}/kit/dev/gsl/include"
    "${USD_ROOT}/include"
    "${CARGO_TARGET_DIR}/cxxbridge"
    "${OGN_GEN_DIR}"
)

target_link_directories(omni_isaacsimrs_bridge PRIVATE "${USD_ROOT}/lib")

target_link_libraries(omni_isaacsimrs_bridge PRIVATE dl "${RUST_CDYLIB_PATH}")

add_custom_command(TARGET omni_isaacsimrs_bridge POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${RUST_CDYLIB_PATH}"
        "${CMAKE_CURRENT_SOURCE_DIR}/bin/"
    COMMENT "Copying Rust cdylib next to plugin .so"
)
