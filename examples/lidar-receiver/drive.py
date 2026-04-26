"""Drive the PublishLidarFlatScanToRust OG node with synthesized inputs.

Kit's --exec runs this script once at startup. The script schedules an
asyncio task that ticks every 100 ms, sets fake inputs on the OG node,
and evaluates the graph. Each evaluation triggers the bridge's
forward_lidar_flatscan which dispatches to whatever consumers are
registered (e.g. the dora publisher loaded via ISAAC_SIM_RS_DORA_RUNNER).

Stand-in for a real RTX upstream (IsaacComputeRTXLidarFlatScan rays
into a USD scene). The rerun-viewer example shows the real-RTX path.
"""

import asyncio
import math

import omni.graph.core as og
import omni.kit.app
import omni.usd

SAMPLES = 360
PERIOD_S = 0.1

omni.usd.get_context().new_stage()

keys = og.Controller.Keys
(graph, nodes, _, _) = og.Controller.edit(
    {"graph_path": "/World/LidarGraph", "evaluator_name": "push"},
    {keys.CREATE_NODES: [("LidarFwd", "omni.isaacsimrs.bridge.PublishLidarFlatScanToRust")]},
)
node = nodes[0]

attrs = {
    name: node.get_attribute(f"inputs:{name}")
    for name in (
        "linearDepthData",
        "intensitiesData",
        "horizontalFov",
        "horizontalResolution",
        "azimuthRange",
        "depthRange",
        "numRows",
        "numCols",
        "rotationRate",
    )
}

og.Controller.set(attrs["horizontalFov"], 360.0)
og.Controller.set(attrs["horizontalResolution"], 360.0 / SAMPLES)
og.Controller.set(attrs["azimuthRange"], [-180.0, 180.0])
og.Controller.set(attrs["depthRange"], [0.1, 30.0])
og.Controller.set(attrs["numRows"], 1)
og.Controller.set(attrs["numCols"], SAMPLES)
og.Controller.set(attrs["rotationRate"], 1.0 / PERIOD_S)


async def drive_loop():
    t = 0.0
    while True:
        depths = [5.0 + math.sin(i * (math.tau / SAMPLES) + t) * 2.0 for i in range(SAMPLES)]
        intensities = [(i + int(t * 25)) % 256 for i in range(SAMPLES)]
        og.Controller.set(attrs["linearDepthData"], depths)
        og.Controller.set(attrs["intensitiesData"], intensities)
        og.Controller.evaluate_sync(graph)
        t += 0.05
        await asyncio.sleep(PERIOD_S)


print("[og_lidar_drive_loop] graph constructed, drive task scheduled")
asyncio.ensure_future(drive_loop())
