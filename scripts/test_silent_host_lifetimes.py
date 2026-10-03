"""Generated object-lifetime regressions; no host service or audio access."""

import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from run_silent_host_probe import require_removed
from silent_host_evidence import EvidenceError
from silent_host_routes import NodeIdentity, RouteError, find_sink, observe_route
from test_silent_host_evidence import PROTOCOL
from test_silent_host_routes import fixture


class StreamLifetimeTests(unittest.TestCase):
    def require_snapshot_removed(self, snapshot):
        clock = SimpleNamespace(now=0.0)

        def sleep(seconds):
            clock.now += seconds

        with patch("run_silent_host_probe.time", SimpleNamespace(
                monotonic=lambda: clock.now, sleep=sleep)):
            require_removed(SimpleNamespace(snapshot=lambda: snapshot),
                            find_sink(fixture(), "owned-test"), 4321,
                            PROTOCOL, io.StringIO(), node=NodeIdentity(30, 3000))

    def test_recycled_node_id_as_metadata_client_is_not_an_orphan_stream(self):
        snapshot = fixture()[:1] + [{
            "id": 30, "type": "PipeWire:Interface:Client",
            "info": {"props": {"object.serial": 4000,
                               "application.process.id": 8765,
                               "application.name": "generated-metadata-client"}},
        }]
        self.require_snapshot_removed(snapshot)

    def test_recycled_node_id_with_new_serial_is_not_the_original_node(self):
        snapshot = fixture()[:1] + [{
            "id": 30, "type": "PipeWire:Interface:Node",
            "info": {"props": {"object.serial": 4000, "media.class": "Audio/Sink",
                               "node.name": "unrelated-generated-sink"}},
        }]
        self.require_snapshot_removed(snapshot)

    def test_missing_object_type_is_not_proof_of_recycled_identity(self):
        with self.assertRaises(RouteError):
            self.require_snapshot_removed(fixture()[:1] + [{"id": 30, "info": {"props": {}}}])

    def test_same_lifetime_without_client_links_or_target_still_blocks_removal(self):
        orphan = fixture()[2]
        orphan["info"]["props"].pop("target.object")
        with self.assertRaisesRegex(EvidenceError, "teardown deadline"):
            self.require_snapshot_removed(fixture()[:1] + [orphan])

    def test_original_serial_under_another_id_is_inconsistent_not_absent(self):
        node = fixture()[2]
        node["id"] = 31
        node["info"]["props"].pop("target.object")
        with self.assertRaisesRegex(RouteError, "identity|serial"):
            self.require_snapshot_removed(fixture()[:1] + [node])

    def test_missing_or_malformed_node_serial_never_establishes_absence(self):
        for value in (None, True, -1, 3.5, "unknown", 2**64):
            with self.subTest(serial=value), self.assertRaises(RouteError):
                node = fixture()[2]
                node["info"]["props"].pop("target.object")
                node["info"]["props"]["object.serial"] = value
                self.require_snapshot_removed(fixture()[:1] + [node])

    def test_replacement_targeting_owned_sink_still_blocks_removal(self):
        node = fixture()[2]
        node["id"] = 31
        node["info"]["props"]["object.serial"] = 4000
        with self.assertRaisesRegex(EvidenceError, "teardown deadline"):
            self.require_snapshot_removed(fixture()[:1] + [node])

    def test_foreign_incoming_link_is_not_excused_by_original_stream_absence(self):
        with self.assertRaisesRegex(RouteError, "foreign"):
            self.require_snapshot_removed(fixture()[:1] + [fixture()[-1]])

    def test_route_admission_requires_original_node_serial(self):
        for value in (None, True, -1, 3.5, "unknown", 2**64):
            with self.subTest(serial=value), self.assertRaises(RouteError):
                snapshot = fixture()
                snapshot[2]["info"]["props"]["object.serial"] = value
                observe_route(snapshot, find_sink(snapshot, "owned-test"), 4321)


if __name__ == "__main__":
    unittest.main()
