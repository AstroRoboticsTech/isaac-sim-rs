cmake_minimum_required(VERSION 3.20)
project(isaac_sim_rs_link_smoke LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Walk up to the workspace root and locate cargo's target/ output. The
# caller (run_cpp_link_smoke.sh) runs `cargo build -p isaac-sim-bridge`
# first so the .so + cxxbridge shim are guaranteed to exist.
set(WORKSPACE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../..")
set(CARGO_TARGET_DIR "$ENV{CARGO_TARGET_DIR}" CACHE PATH "Cargo target directory")
if(NOT CARGO_TARGET_DIR)
    set(CARGO_TARGET_DIR "${WORKSPACE_ROOT}/target" CACHE PATH "Cargo target directory" FORCE)
endif()
if(DEFINED ENV{CARGO_PROFILE})
    set(CARGO_PROFILE "$ENV{CARGO_PROFILE}" CACHE STRING "Cargo build profile" FORCE)
else()
    set(CARGO_PROFILE "release" CACHE STRING "Cargo build profile")
endif()

set(BRIDGE_CDYLIB "${CARGO_TARGET_DIR}/${CARGO_PROFILE}/libisaac_sim_bridge.so")
if(NOT EXISTS "${BRIDGE_CDYLIB}")
    message(FATAL_ERROR "Rust cdylib not found at ${BRIDGE_CDYLIB}; run 'cargo build -p isaac-sim-bridge --${CARGO_PROFILE}' first")
endif()

set(CXX_BRIDGE_SHIM "${CARGO_TARGET_DIR}/cxxbridge/isaac-sim-bridge/src/lib.rs.cc")
if(NOT EXISTS "${CXX_BRIDGE_SHIM}")
    message(FATAL_ERROR "cxxbridge shim not found at ${CXX_BRIDGE_SHIM}; run 'cargo build -p isaac-sim-bridge --${CARGO_PROFILE}' first")
endif()

file(GLOB CXX_RUNTIME_CC "$ENV{HOME}/.cargo/registry/src/*/cxx-*/src/cxx.cc")
if(NOT CXX_RUNTIME_CC)
    message(FATAL_ERROR "cxx.cc not in cargo registry; build the bridge first")
endif()
list(GET CXX_RUNTIME_CC 0 CXX_RUNTIME_CC)

add_executable(link_smoke
    driver.cpp
    "${CXX_BRIDGE_SHIM}"
    "${CXX_RUNTIME_CC}"
)

target_include_directories(link_smoke PRIVATE "${CARGO_TARGET_DIR}/cxxbridge")

target_link_directories(link_smoke PRIVATE "${CARGO_TARGET_DIR}/${CARGO_PROFILE}")
target_link_libraries(link_smoke PRIVATE -l:libisaac_sim_bridge.so)

set_target_properties(link_smoke PROPERTIES
    BUILD_RPATH "${CARGO_TARGET_DIR}/${CARGO_PROFILE}"
    INSTALL_RPATH "${CARGO_TARGET_DIR}/${CARGO_PROFILE}"
)
