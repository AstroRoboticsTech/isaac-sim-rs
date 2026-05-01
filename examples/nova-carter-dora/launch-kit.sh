#!/bin/bash
set -euo pipefail

: "${ISAAC_SIM:?ISAAC_SIM env var not set; export it before running 'dora start'}"
: "${ISAAC_SIM_RS:?ISAAC_SIM_RS env var not set; export it before running 'dora start'}"

# dlopen target for the bridge plugin's lifecycle: this is the dora
# runner cdylib. It opens a DoraNode + EventStream from the env vars
# dora supplies and registers one publisher per sensor + a cmd_vel
# subscriber.
export ISAAC_SIM_RS_DORA_RUNNER="${ISAAC_SIM_RS}/cpp/omni.isaacsimrs.bridge/bin/libisaac_sim_dora.so"

# Per-sensor SOURCE filters tie the publisher to a specific prim path
# in the Carter scene; OUTPUT names match dataflow.yml.
export ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_SOURCE="${ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_SOURCE:-/Root/World/Carter/chassis_link/lidar_2d}"
export ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT="${ISAAC_SIM_RS_DORA_LIDAR_FLATSCAN_OUTPUT:-lidar_flatscan}"
export ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_SOURCE="${ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_SOURCE:-/Root/World/Carter/chassis_link/sensors/XT_32/PandarXT_32_10hz}"
export ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_OUTPUT="${ISAAC_SIM_RS_DORA_LIDAR_POINTCLOUD_OUTPUT:-lidar_pointcloud}"
export ISAAC_SIM_RS_DORA_CAMERA_RGB_SOURCE="${ISAAC_SIM_RS_DORA_CAMERA_RGB_SOURCE:-/Root/World/Carter/chassis_link/camera_rgb}"
export ISAAC_SIM_RS_DORA_CAMERA_RGB_OUTPUT="${ISAAC_SIM_RS_DORA_CAMERA_RGB_OUTPUT:-camera_rgb}"
export ISAAC_SIM_RS_DORA_CAMERA_DEPTH_SOURCE="${ISAAC_SIM_RS_DORA_CAMERA_DEPTH_SOURCE:-/Root/World/Carter/chassis_link/camera_rgb}"
export ISAAC_SIM_RS_DORA_CAMERA_DEPTH_OUTPUT="${ISAAC_SIM_RS_DORA_CAMERA_DEPTH_OUTPUT:-camera_depth}"
export ISAAC_SIM_RS_DORA_CAMERA_INFO_SOURCE="${ISAAC_SIM_RS_DORA_CAMERA_INFO_SOURCE:-/Root/World/Carter/chassis_link/camera_rgb}"
export ISAAC_SIM_RS_DORA_CAMERA_INFO_OUTPUT="${ISAAC_SIM_RS_DORA_CAMERA_INFO_OUTPUT:-camera_info}"
export ISAAC_SIM_RS_DORA_IMU_SOURCE="${ISAAC_SIM_RS_DORA_IMU_SOURCE:-/Root/World/Carter/chassis_link/imu}"
export ISAAC_SIM_RS_DORA_IMU_OUTPUT="${ISAAC_SIM_RS_DORA_IMU_OUTPUT:-imu}"
export ISAAC_SIM_RS_DORA_ODOMETRY_SOURCE="${ISAAC_SIM_RS_DORA_ODOMETRY_SOURCE:-/Root/World/Carter/chassis_link}"
export ISAAC_SIM_RS_DORA_ODOMETRY_OUTPUT="${ISAAC_SIM_RS_DORA_ODOMETRY_OUTPUT:-odometry}"

# Publisher direction (bridge → dora): SOURCE is the prim-path filter;
# OUTPUT is the dora node output id. The output default is "cmd_vel_observed"
# (not "cmd_vel") so it never collides with the subscriber INPUT below.
export ISAAC_SIM_RS_DORA_CMD_VEL_SOURCE="${ISAAC_SIM_RS_DORA_CMD_VEL_SOURCE:-/Root/World/Carter}"
export ISAAC_SIM_RS_DORA_CMD_VEL_OUTPUT="${ISAAC_SIM_RS_DORA_CMD_VEL_OUTPUT:-cmd_vel_observed}"

# Subscriber direction (dora → bridge): INPUT is the dora input id the
# node listens on; TARGET is the producer-slot key (articulation prim path)
# that the C++ ApplyCmdVelFromRust node polls every OG tick.
export ISAAC_SIM_RS_DORA_CMD_VEL_INPUT="${ISAAC_SIM_RS_DORA_CMD_VEL_INPUT:-cmd_vel}"
export ISAAC_SIM_RS_DORA_CMD_VEL_TARGET="${ISAAC_SIM_RS_DORA_CMD_VEL_TARGET:-/Root/World/Carter}"

exec "${ISAAC_SIM}/kit/kit" \
    "${ISAAC_SIM}/apps/isaacsim.exp.full.kit" \
    --no-window \
    --no-ros-env \
    --ext-folder "${ISAAC_SIM_RS}/cpp" \
    --enable omni.isaacsimrs.bridge \
    --enable isaacsim.sensors.rtx \
    --enable isaacsim.robot.wheeled_robots \
    --exec "${ISAAC_SIM_RS}/examples/nova-carter/drive.py"
