cmake_minimum_required(VERSION 3.20)
project(omni_isaacsimrs_bridge LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    set(EXT_PLATFORM "linux-x86_64")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Windows")
    set(EXT_PLATFORM "windows-x86_64")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
    set(EXT_PLATFORM "macos-universal")
else()
    message(FATAL_ERROR "Unsupported platform: ${CMAKE_SYSTEM_NAME}")
endif()

# Each Ogn codegen step launches a headless Kit process that locks
# packman caches and Kit shared-mem keys; running them in parallel
# races on those locks. Pin to one job at a time.
set_property(GLOBAL PROPERTY JOB_POOLS ogn_codegen=1)

find_package(CUDAToolkit REQUIRED)

if(NOT DEFINED ENV{ISAAC_SIM_PATH})
    message(FATAL_ERROR "ISAAC_SIM_PATH env var must point to the Isaac Sim install root (the directory containing kit/dev/include)")
endif()
set(ISAAC_SIM "$ENV{ISAAC_SIM_PATH}")

set(CARGO_TARGET_DIR "$ENV{CARGO_TARGET_DIR}" CACHE PATH "Cargo target directory")
if(NOT CARGO_TARGET_DIR)
    set(CARGO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target" CACHE PATH "Cargo target directory" FORCE)
endif()
if(DEFINED ENV{CARGO_PROFILE})
    set(CARGO_PROFILE "$ENV{CARGO_PROFILE}" CACHE STRING "Cargo build profile (debug or release)" FORCE)
else()
    set(CARGO_PROFILE "debug" CACHE STRING "Cargo build profile (debug or release)")
endif()
set(CARGO_OUTPUT_DIR "${CARGO_TARGET_DIR}/${CARGO_PROFILE}")

# cargo invokes cargo on every build. The custom target has no
# declared sources or outputs cmake can introspect — that's
# intentional, since cargo's own incremental machinery is the source
# of truth for "what changed". cmake re-runs cargo every build (a
# no-op in cargo when nothing changed; ~ms cost). The plugin target's
# `add_dependencies` below makes cargo run *before* the plugin links,
# so the cdylib + cxxbridge shim files at ${CARGO_OUTPUT_DIR} and
# ${CARGO_TARGET_DIR}/cxxbridge are guaranteed current at link time.
# This is what fixes the "stale cdylib" + "stale cxxbridge shim"
# class of bugs we kept hitting.
set(CARGO_PROFILE_FLAG "")
if(CARGO_PROFILE STREQUAL "release")
    set(CARGO_PROFILE_FLAG "--release")
endif()
add_custom_target(cargo_cdylibs ALL
    COMMAND ${CMAKE_COMMAND} -E env
        "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
        cargo build --workspace --lib ${CARGO_PROFILE_FLAG}
            --features isaac-sim-bridge/cdylib
            --features isaac-sim-dora/cdylib
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../.."
    BYPRODUCTS
        "${CARGO_OUTPUT_DIR}/libisaac_sim_bridge.so"
        "${CARGO_OUTPUT_DIR}/libisaac_sim_dora.so"
        "${CARGO_OUTPUT_DIR}/libexample_nova_carter.so"
    COMMENT "[isaacsimrs] cargo build --workspace --lib"
    USES_TERMINAL
)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(Ogn)
include(Packman)

# USD headers come in via Packman because OmniGraph's
# `omni/graph/core/StringUtils.h` (in kit/dev/include) includes
# `pxr/base/tf/token.h` transitively. The plugin .so does NOT link
# against any libusd_* / libpxr_* — confirmed via `readelf -d ... |
# grep NEEDED`. So we use ${USD_ROOT}/include below but skip the
# corresponding lib directory.
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

file(GLOB CXX_RUNTIME_CC "$ENV{HOME}/.cargo/registry/src/*/cxx-*/src/cxx.cc")
if(NOT CXX_RUNTIME_CC)
    message(FATAL_ERROR "Could not find cxx.cc in cargo registry; ensure cargo has been run at least once before cmake configure")
endif()
list(GET CXX_RUNTIME_CC 0 CXX_RUNTIME_CC)

add_library(omni_isaacsimrs_bridge SHARED
    ${PLUGIN_SOURCES}
    ${NODE_SOURCES}
    ${OGN_GENERATED_HEADERS}
    ${CXX_BRIDGE_SHIMS}
    "${CXX_RUNTIME_CC}"
)

# Plugin can't link until cargo has produced the cdylibs (and the
# cxxbridge shim files in ${CARGO_TARGET_DIR}/cxxbridge/...). This is
# the dependency that fixes the stale-cdylib failure modes — without
# it, cmake would happily link against whatever ./bin/ already had.
add_dependencies(omni_isaacsimrs_bridge cargo_cdylibs)

set_target_properties(omni_isaacsimrs_bridge PROPERTIES
    OUTPUT_NAME "omni.isaacsimrs.bridge.plugin"
    PREFIX "lib"
    SUFFIX ".so"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/bin/${EXT_PLATFORM}"
    BUILD_RPATH "$ORIGIN"
    INSTALL_RPATH "$ORIGIN"
    INSTALL_RPATH_USE_LINK_PATH OFF
    BUILD_WITH_INSTALL_RPATH ON
)

target_include_directories(omni_isaacsimrs_bridge PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${ISAAC_SIM}/kit/dev/include"
    "${ISAAC_SIM}/kit/dev/fabric/include"
    "${ISAAC_SIM}/kit/dev/gsl/include"
    "${USD_ROOT}/include"
    "${CARGO_TARGET_DIR}/cxxbridge"
    "${OGN_GEN_DIR}"
)

# Link by absolute path so cmake tracks the cdylib's mtime as a link
# input — when cargo rebuilds it, cmake notices and relinks the
# plugin (which then triggers the POST_BUILD copy below). The runtime
# DT_NEEDED entry stays as bare `libisaac_sim_bridge.so` because
# that's the file's basename, and `$ORIGIN` in RUNPATH resolves it
# against bin/ at dlopen time.
target_link_libraries(omni_isaacsimrs_bridge PRIVATE
    dl
    "${CARGO_OUTPUT_DIR}/libisaac_sim_bridge.so"
    CUDA::cudart
)

# Copy each cdylib next to the plugin .so. POST_BUILD here is fine —
# the plugin always relinks when any cdylib changes (because the
# cargo_cdylibs dependency forces cargo first, and cmake recognises
# the new mtime via BYPRODUCTS).
foreach(crate isaac_sim_bridge isaac_sim_dora example_nova_carter)
    add_custom_command(TARGET omni_isaacsimrs_bridge POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${CARGO_OUTPUT_DIR}/lib${crate}.so"
            "${CMAKE_CURRENT_SOURCE_DIR}/bin/${EXT_PLATFORM}/"
        COMMENT "[isaacsimrs] copying lib${crate}.so to bin/${EXT_PLATFORM}/"
    )
endforeach()

install(TARGETS omni_isaacsimrs_bridge
    LIBRARY DESTINATION "omni.isaacsimrs.bridge/bin/${EXT_PLATFORM}"
)
install(FILES
    "${CARGO_OUTPUT_DIR}/libisaac_sim_bridge.so"
    "${CARGO_OUTPUT_DIR}/libisaac_sim_dora.so"
    DESTINATION "omni.isaacsimrs.bridge/bin/${EXT_PLATFORM}"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/config/"
    DESTINATION "omni.isaacsimrs.bridge/config"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/nodes/"
    DESTINATION "omni.isaacsimrs.bridge/nodes"
    FILES_MATCHING PATTERN "*.ogn"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/docs/"
    DESTINATION "omni.isaacsimrs.bridge/docs"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/data/"
    DESTINATION "omni.isaacsimrs.bridge/data"
    FILES_MATCHING PATTERN "*.png"
)
