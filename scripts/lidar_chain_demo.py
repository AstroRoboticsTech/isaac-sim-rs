"""Wires PublishLidarToRust to IsaacComputeRTXLidarFlatScan in an OmniGraph.

This is the structural-only demo: with no upstream RTX render product, the
FlatScan node produces empty outputs and our forwarder receives empty
buffers. The point is to prove the connection succeeds without schema
errors -- that PublishLidarToRust accepts exactly the outputs that
IsaacComputeRTXLidarFlatScan produces.

Run via:
    $ISAAC_SIM/kit/kit $ISAAC_SIM/apps/isaacsim.exp.base.kit \\
        --no-window --no-ros-env \\
        --ext-folder cpp \\
        --enable omni.isaacsimrs.bridge \\
        --enable isaacsim.sensors.rtx \\
        --exec scripts/lidar_chain_demo.py
"""

import omni.graph.core as og
import omni.kit.app
import omni.usd

print("DEMO: starting")
omni.usd.get_context().new_stage()

keys = og.Controller.Keys
try:
    (graph, nodes, _, _) = og.Controller.edit(
        {"graph_path": "/World/LidarGraph", "evaluator_name": "push"},
        {
            keys.CREATE_NODES: [
                ("FlatScan", "isaacsim.sensors.rtx.IsaacComputeRTXLidarFlatScan"),
                ("PublishToRust", "omni.isaacsimrs.bridge.PublishLidarToRust"),
            ],
            keys.CONNECT: [
                ("FlatScan.outputs:exec", "PublishToRust.inputs:exec"),
                (
                    "FlatScan.outputs:linearDepthData",
                    "PublishToRust.inputs:linearDepthData",
                ),
                (
                    "FlatScan.outputs:intensitiesData",
                    "PublishToRust.inputs:intensitiesData",
                ),
                (
                    "FlatScan.outputs:horizontalFov",
                    "PublishToRust.inputs:horizontalFov",
                ),
                (
                    "FlatScan.outputs:horizontalResolution",
                    "PublishToRust.inputs:horizontalResolution",
                ),
                ("FlatScan.outputs:azimuthRange", "PublishToRust.inputs:azimuthRange"),
                ("FlatScan.outputs:depthRange", "PublishToRust.inputs:depthRange"),
                ("FlatScan.outputs:numRows", "PublishToRust.inputs:numRows"),
                ("FlatScan.outputs:numCols", "PublishToRust.inputs:numCols"),
                ("FlatScan.outputs:rotationRate", "PublishToRust.inputs:rotationRate"),
            ],
        },
    )

    flat_scan, publish = nodes
    print(f"DEMO: FlatScan at {flat_scan.get_prim_path()}")
    print(f"DEMO: PublishToRust at {publish.get_prim_path()}")
    print(
        "DEMO: connections established between FlatScan outputs and PublishToRust inputs"
    )

    og.Controller.evaluate_sync(graph)
    print("DEMO: graph evaluated cleanly")
    print("DEMO: PASS")
except Exception as e:
    import traceback

    traceback.print_exc()
    print(f"DEMO: FAIL - {type(e).__name__}: {e}")

omni.kit.app.get_app().post_quit()
