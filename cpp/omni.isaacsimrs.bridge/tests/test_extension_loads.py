# SPDX-License-Identifier: MPL-2.0

"""Minimal smoke test: extension loads, OG node types are registered."""

import omni.kit.test


class TestExtensionLoads(omni.kit.test.AsyncTestCaseFailOnLogError):
    async def test_extension_enabled(self):
        import omni.kit.app
        manager = omni.kit.app.get_app().get_extension_manager()
        ext_id = manager.get_enabled_extension_id("omni.isaacsimrs.bridge")
        self.assertIsNotNone(ext_id, "omni.isaacsimrs.bridge not enabled")

    async def test_og_nodes_registered(self):
        import omni.graph.core as og
        registry = og.get_registry()
        for node_type in [
            "omni.isaacsimrs.bridge.PublishLidarFlatScanToRust",
            "omni.isaacsimrs.bridge.PublishLidarPointCloudToRust",
            "omni.isaacsimrs.bridge.PublishCameraRgbToRust",
            "omni.isaacsimrs.bridge.PublishCameraDepthToRust",
            "omni.isaacsimrs.bridge.PublishCameraInfoToRust",
            "omni.isaacsimrs.bridge.PublishImuToRust",
            "omni.isaacsimrs.bridge.PublishOdometryToRust",
            "omni.isaacsimrs.bridge.ApplyCmdVelFromRust",
        ]:
            self.assertTrue(
                registry.get_node_type(node_type),
                f"{node_type} not registered",
            )
