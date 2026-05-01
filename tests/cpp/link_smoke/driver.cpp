// SPDX-License-Identifier: MPL-2.0
// Tier-(b) link smoke test: verifies the cxx::bridge ABI between the
// C++ side and libisaac_sim_bridge.so without Kit, USD, or a GPU. If
// this links and runs to exit 0, the cdylib's exported symbols + slice
// layout + str passing all line up with what the Carb plugin will see.
//
// Catches:
//   - cxx::bridge symbol drift (e.g. signature mismatch after editing
//     the bridge mod without rebuilding the cdylib)
//   - undefined references in the generated lib.rs.cc shim
//   - Slice<const T> / Str layout assumptions
//
// Doesn't catch:
//   - OGN schema bugs (covered by tests/ogn_schema.rs)
//   - Kit lifecycle issues (covered by tier-(c) Kit smoke)
//   - GPU memcpy correctness (covered by tier-(c) Kit smoke)

#include "isaac-sim-bridge/src/lib.rs.h"

#include <atomic>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <iostream>

int main() {
    // Bring the Rust side up — registers default consumers in
    // lifecycle.rs::register_default_consumers().
    isaacsimrs::init();

    // FlatScan path: 4-arg dispatch with depths + intensities.
    {
        const float depths[] = {0.5f, 1.2f, 2.7f, 3.0f};
        const std::uint8_t intensities[] = {10, 50, 200, 100};
        const char* src = "/link-smoke/flatscan";
        rust::Str source{src, std::strlen(src)};
        rust::Slice<const float> scan{depths, 4};
        rust::Slice<const std::uint8_t> intens{intensities, 4};
        isaacsimrs::LidarFlatScanMeta meta{
            270.0f, 0.25f, -135.0f, 135.0f, 0.1f, 30.0f, 1, 4, 10.0f
        };
        isaacsimrs::forward_lidar_flatscan(source, scan, intens, meta);
    }

    // PointCloud path: packed XYZ buffer.
    {
        const float points[] = {
            1.0f, 0.0f, 0.0f,
            0.0f, 1.0f, 0.0f,
            0.0f, 0.0f, 1.0f,
        };
        const char* src = "/link-smoke/pointcloud";
        rust::Str source{src, std::strlen(src)};
        rust::Slice<const float> slice{points, 9};
        isaacsimrs::LidarPointCloudMeta meta{3, 3, 1};
        isaacsimrs::forward_lidar_pointcloud(source, slice, meta);
    }

    // CmdVel poll path: Rust→C++ direction. With no producer registered,
    // poll must return false and not write to the out param.
    {
        const char* target = "/link-smoke/cmd_vel/never_registered";
        rust::Str target_id{target, std::strlen(target)};
        isaacsimrs::CmdVel out{};
        bool hit = isaacsimrs::poll_cmd_vel(target_id, out);
        assert(!hit && "poll_cmd_vel returned true for unregistered target");
    }

    // double_value: trivial scalar round-trip — useful as a sanity ABI
    // check independent of slice/str.
    {
        std::int32_t v = isaacsimrs::double_value(21);
        assert(v == 42 && "double_value(21) != 42");
    }

    std::cout << "[link-smoke] all FFI paths returned cleanly" << std::endl;
    return 0;
}
