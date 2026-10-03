"""Generated metadata only: never query PipeWire or start audio."""

import copy
import unittest

from silent_host_routes import RouteError, find_sink, observe_route


def fixture():
    def obj(kind, object_id, props, **info):
        return {"id": object_id, "type": f"PipeWire:Interface:{kind}",
                "info": {"props": props, **info}}

    return [
        obj("Node", 10, {"node.name": "owned-test", "object.serial": 1000,
                         "media.class": "Audio/Sink"}),
        obj("Client", 20, {"application.process.id": 4321}),
        obj("Node", 30, {"client.id": 20, "object.serial": 3000,
                         "media.class": "Stream/Output/Audio",
                         "target.object": "1000", "node.dont-fallback": True,
                         "node.dont-reconnect": True, "node.dont-move": True}),
        obj("Port", 40, {"node.id": 30, "audio.channel": "FL"}),
        obj("Port", 41, {"node.id": 30, "audio.channel": "FR"}),
        obj("Port", 50, {"node.id": 10, "audio.channel": "FL"}),
        obj("Port", 51, {"node.id": 10, "audio.channel": "FR"}),
        obj("Link", 60, {}, **{"output-node-id": 30, "input-node-id": 10,
                              "output-port-id": 40, "input-port-id": 50,
                              "state": "active"}),
        obj("Link", 61, {}, **{"output-node-id": 30, "input-node-id": 10,
                              "output-port-id": 41, "input-port-id": 51,
                              "state": "active"}),
    ]


class RouteAdmissionTests(unittest.TestCase):
    def test_exact_process_stereo_links_bind_to_owned_sink_serial(self):
        snapshot = fixture()
        sink = find_sink(snapshot, "owned-test")
        route = observe_route(snapshot, sink, 4321)
        self.assertEqual(route, {"client_id": 20, "node_id": 30,
                                 "link_ids": [60, 61]})
        self.assertIsNone(observe_route(snapshot[:1], sink, 4321))

    def test_numeric_truthiness_is_not_a_boolean_safety_property(self):
        snapshot = fixture()
        sink = find_sink(snapshot, "owned-test")
        snapshot[2]["info"]["props"]["node.dont-fallback"] = 1
        with self.assertRaisesRegex(RouteError, "safety property"):
            observe_route(snapshot, sink, 4321)

    def test_safe_incomplete_startup_links_wait_for_bounded_admission(self):
        snapshot = fixture()
        sink = find_sink(snapshot, "owned-test")
        self.assertIsNone(observe_route(snapshot[:-1], sink, 4321))
        snapshot[-1]["info"]["state"] = "paused"
        self.assertIsNone(observe_route(snapshot, sink, 4321))

    def test_unsafe_identity_routing_properties_and_channel_mutations_reject(self):
        def set_prop(snapshot, key, value):
            snapshot[2]["info"]["props"][key] = value

        mutations = [
            lambda s: set_prop(s, "target.object", "10"),
            lambda s: set_prop(s, "node.dont-fallback", False),
            lambda s: set_prop(s, "node.dont-reconnect", "false"),
            lambda s: set_prop(s, "node.dont-move", None),
            lambda s: set_prop(s, "media.class", "Stream/Input/Audio"),
            lambda s: s[0]["info"]["props"].update({"object.serial": 1001}),
            lambda s: s[-1]["info"].update({"input-node-id": 999}),
            lambda s: s[-1]["info"].update({"output-node-id": 999}),
            lambda s: s[-1]["info"].update({"output-node-id": 10}),
            lambda s: s[-1]["info"].update({"input-port-id": 50}),
            lambda s: s[-1]["info"].update({"output-port-id": 40,
                                          "input-port-id": 50}),
            lambda s: s[-1]["info"].update({"state": "error"}),
            lambda s: s.append(copy.deepcopy(s[0])),
            lambda s: s[2].update({"info": None}),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(mutation=index):
                snapshot = fixture()
                sink = find_sink(snapshot, "owned-test")
                mutate(snapshot)
                with self.assertRaises(RouteError):
                    observe_route(snapshot, sink, 4321)

    def test_wrong_pid_and_ambiguous_process_nodes_never_get_admitted(self):
        snapshot = fixture()
        sink = find_sink(snapshot, "owned-test")
        with self.assertRaisesRegex(RouteError, "foreign"):
            observe_route(snapshot, sink, 9876)
        duplicate = copy.deepcopy(snapshot[2])
        duplicate["id"] = 31
        snapshot.append(duplicate)
        with self.assertRaisesRegex(RouteError, "ambiguous"):
            observe_route(snapshot, sink, 4321)

    def test_serials_and_boolean_properties_have_supported_text_form(self):
        snapshot = fixture()
        for key in ("node.dont-fallback", "node.dont-reconnect", "node.dont-move"):
            snapshot[2]["info"]["props"][key] = "true"
        snapshot[2]["info"]["props"]["client.id"] = "20"
        snapshot[0]["info"]["props"]["object.serial"] = "1000"
        route = observe_route(snapshot, find_sink(snapshot, "owned-test"), 4321)
        self.assertEqual(route["node_id"], 30)

    def test_absent_sink_or_corrupt_snapshot_is_never_pending(self):
        sink = find_sink(fixture(), "owned-test")
        for snapshot in (None, {}, [], [{"type": "broken"}], [{"id": False}]):
            with self.subTest(snapshot=snapshot), self.assertRaises(RouteError):
                observe_route(snapshot, sink, 4321)


if __name__ == "__main__":
    unittest.main()
