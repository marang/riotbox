"""Generated Pulse module metadata only; no host command is executed."""

import unittest
from unittest.mock import patch

from silent_host_modules import owned_modules, remove_owned_sink
from silent_host_evidence import EvidenceError


QUERY = ["pactl", "--format=text", "list", "short", "modules"]
JSON_QUERY = ["pactl", "--format=json", "list", "short", "modules"]
ARGUMENT = "sink_name=owned-test channels=2 channel_map=front-left,front-right"
OWNED = f"42\tmodule-null-sink\t{ARGUMENT}\t\n"
FOREIGN = "7\tmodule-always-sink\t\t\n"
PROTOCOL = {"teardown_deadline_seconds": 5, "route_poll_interval_ms": 100}


class GeneratedHost:
    def __init__(self, modules=OWNED):
        self.modules = modules
        self.commands = []
        self.nodes = []
        self.snapshot_count = 0

    def json(self, command):
        self.commands.append(command)
        if command != JSON_QUERY:
            raise AssertionError(f"unexpected generated command: {command}")
        # Compatibility adapter for the old literal single-line fixtures only.
        # Independent V3 tests supply explicit JSON and multiline text separately.
        if not isinstance(self.modules, str):
            return []
        return [{"name": fields[1], "argument": fields[2]}
                for line in self.modules.splitlines()
                if len(fields := line.split("\t")) >= 3 and fields[1]]

    def text(self, command):
        self.commands.append(command)
        if command == QUERY:
            return self.modules
        if command == ["pactl", "unload-module", "42"]:
            self.modules = "\n".join(line for line in self.modules.splitlines()
                                     if not line.startswith("42\t"))
            self.nodes = []
            return ""
        raise AssertionError(f"unexpected generated command: {command}")

    def snapshot(self):
        self.snapshot_count += 1
        return self.nodes


class OwnedModulesTests(unittest.TestCase):
    def test_one_large_pulse_token_checks_deadline_during_lexing(self):
        host = GeneratedHost("7\tmodule-always-sink\t\"" + "x" * 100000 + "\"\t\n")
        ticks = [0]

        def clock():
            ticks[0] += 1
            return 5.0 if ticks[0] >= 100 else 0.0

        with patch("silent_host_modules.time.monotonic", side_effect=clock):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                owned_modules(host, "owned-test", deadline=5.0)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY])

    def test_short_text_supplies_real_index_and_accepts_optional_empty_column(self):
        for row in [OWNED, OWNED.replace("\t\n", "\n")]:
            with self.subTest(row=row):
                host = GeneratedHost("7\tmodule-always-sink\t\t\n" + row)
                self.assertEqual(owned_modules(host, "owned-test"), [
                    {"index": 42, "name": "module-null-sink", "argument": ARGUMENT},
                ])
                self.assertEqual(host.commands, [JSON_QUERY, QUERY])

    def test_owned_identity_requires_exact_type_stereo_arguments_and_no_conflicts(self):
        for row in [
            OWNED.replace("module-null-sink", "module-remap-sink"),
            OWNED.replace(" channels=2", ""),
            OWNED.replace("channels=2", "channels=1"),
            OWNED.replace("front-left,front-right", "front-right,front-left"),
            OWNED.replace(ARGUMENT, ARGUMENT + " rate=48000"),
            OWNED.replace(ARGUMENT, ARGUMENT + " sink_name=other"),
            OWNED.replace(ARGUMENT, "sink_name=other " + ARGUMENT),
            OWNED.replace(ARGUMENT, ARGUMENT + " sink_name=owned-test"),
            OWNED.replace(ARGUMENT, ARGUMENT + " channels=2"),
            OWNED + OWNED.replace("42\t", "43\t"),
        ]:
            with self.subTest(row=row):
                with self.assertRaises(EvidenceError):
                    owned_modules(GeneratedHost(row), "owned-test")

    def test_malformed_columns_ids_and_duplicate_indices_fail_closed(self):
        rows = [OWNED.replace("42\t", value + "\t", 1) for value in (
            "", "042", "+42", "-1", "42.0", "0x2a", "４２", "4294967295", "4294967296", "9" * 30,
        )]
        rows.extend([
            "42\tmodule-null-sink\n", "42 module-null-sink " + ARGUMENT,
            OWNED.replace("\t\n", "\tnonempty\n"), OWNED.replace("\t\n", "\t\t\n"),
            OWNED + "42\tmodule-always-sink\t\t\n", "7\t\t\n", OWNED + "\n",
            OWNED.replace(ARGUMENT, "'" + ARGUMENT), None, [],
        ])
        for row in rows:
            with self.subTest(row=row):
                with self.assertRaises(EvidenceError):
                    owned_modules(GeneratedHost(row), "owned-test")

    def test_uint32_module_indices_and_empty_table_have_literal_controls(self):
        for index in (0, 4294967294):
            host = GeneratedHost(OWNED.replace("42\t", f"{index}\t"))
            self.assertEqual(owned_modules(host, "owned-test")[0]["index"], index)
        self.assertEqual(owned_modules(GeneratedHost(""), "owned-test"), [])

    def test_names_are_argument_values_not_substrings_or_quoted_property_contents(self):
        for argument in [
            ARGUMENT.replace("owned-test", "owned-test-other"),
            ARGUMENT.replace("owned-test", "other-owned-test"),
            'sink_properties="device.description=sink_name=owned-test"',
            'sink_properties="a sink_name=owned-test b"',
        ]:
            with self.subTest(argument=argument):
                self.assertEqual(owned_modules(GeneratedHost(OWNED.replace(ARGUMENT, argument)), "owned-test"), [])
        argument = "channel_map=front-left,front-right sink_name='owned-test' channels=2"
        matched = owned_modules(GeneratedHost(OWNED.replace(ARGUMENT, argument)), "owned-test")
        self.assertEqual(matched, [{"index": 42, "name": "module-null-sink", "argument": argument}])


class ModuleCleanupTests(unittest.TestCase):
    def test_exact_owned_module_is_unloaded_once_with_or_without_load_ack(self):
        for module_id in (42, None):
            with self.subTest(module_id=module_id):
                host = GeneratedHost(FOREIGN + OWNED)
                remove_owned_sink(host, "owned-test", module_id, PROTOCOL)
                self.assertEqual(host.commands, [JSON_QUERY, QUERY, ["pactl", "unload-module", "42"], JSON_QUERY, QUERY])
                self.assertEqual(host.snapshot_count, 1)
                self.assertEqual(host.modules, FOREIGN.rstrip("\n"))

    def test_observed_replacement_at_bound_id_is_never_unloaded_or_called_absent(self):
        for rows in [
            OWNED.replace("owned-test", "other"),
            "42\tmodule-always-sink\t\t\n",
            OWNED.replace("module-null-sink", "module-remap-sink"),
            OWNED.replace("42\t", "43\t"),
            "42\tmodule-always-sink\t\t\n" + OWNED.replace("42\t", "43\t"),
        ]:
            with self.subTest(rows=rows):
                host = GeneratedHost(rows)
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", 42, PROTOCOL)
                self.assertEqual(host.commands, [JSON_QUERY, QUERY])

    def test_post_unload_replacement_is_rejected_without_a_second_unload(self):
        class ReplacedHost(GeneratedHost):
            def text(self, command):
                result = super().text(command)
                if command == ["pactl", "unload-module", "42"]:
                    self.modules = "42\tmodule-always-sink\t\t\n"
                return result

        host = ReplacedHost()
        with self.assertRaises(EvidenceError):
            remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY, ["pactl", "unload-module", "42"], JSON_QUERY, QUERY])

    def test_first_query_consumes_the_same_deadline_before_any_unload_or_snapshot(self):
        for rows in (OWNED, FOREIGN):
            with self.subTest(rows=rows):
                elapsed = [0.0]

                class LateHost(GeneratedHost):
                    def text(self, command):
                        result = super().text(command)
                        if command == QUERY:
                            elapsed[0] += 5.0
                        return result

                host = LateHost(rows)
                with patch("silent_host_modules.time.monotonic", side_effect=lambda: elapsed[0]):
                    with self.assertRaisesRegex(EvidenceError, "deadline"):
                        remove_owned_sink(host, "owned-test", 42, PROTOCOL)
                self.assertEqual(host.commands, [JSON_QUERY, QUERY])
                self.assertEqual(host.snapshot_count, 0)

    def test_unload_time_cannot_grant_fresh_cleanup_queries_after_deadline(self):
        elapsed = [0.0]

        class SlowHost(GeneratedHost):
            def text(self, command):
                result = super().text(command)
                elapsed[0] += 2.5
                return result

        host = SlowHost()
        with patch("silent_host_modules.time.monotonic", side_effect=lambda: elapsed[0]):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                remove_owned_sink(host, "owned-test", 42, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY, ["pactl", "unload-module", "42"]])
        self.assertEqual(host.snapshot_count, 0)

    def test_invalid_bound_ids_fail_before_query_or_unload(self):
        for module_id in (True, False, "42", 42.0, -1, 4294967295, 4294967296):
            with self.subTest(module_id=module_id):
                host = GeneratedHost()
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", module_id, PROTOCOL)
                self.assertEqual(host.commands, [])
                self.assertEqual(host.snapshot_count, 0)

    def test_missing_ack_never_selects_an_ambiguous_or_inexact_owner(self):
        for rows in (OWNED + OWNED.replace("42\t", "43\t"),
                     OWNED.replace("channels=2", "channels=1")):
            with self.subTest(rows=rows):
                host = GeneratedHost(rows)
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", None, PROTOCOL)
                self.assertEqual(host.commands, [JSON_QUERY, QUERY])

    def test_absence_requires_modules_and_nodes_but_never_unloads_a_foreign_sink(self):
        host = GeneratedHost(OWNED.replace("owned-test", "unrelated"))
        remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY])
        self.assertEqual(host.snapshot_count, 1)

    def test_malformed_sink_metadata_never_proves_absence(self):
        for snapshot in (
            {}, "", [None], [{"type": "PipeWire:Interface:Node"}],
            [{"id": 10}], [{"id": 10, "type": "PipeWire:Interface:"}],
            [{"id": 10, "type": "PipeWire:Interface:Node"}],
            [{"id": 10, "type": "PipeWire:Interface:Node", "info": {}}],
            [{"id": 10, "type": "PipeWire:Interface:Node", "info": {"props": []}}],
            [{"id": 10, "type": "PipeWire:Interface:Node", "info": {"props": {"node.name": ""}}}],
            [{"id": 10, "type": "PipeWire:Interface:Node", "info": {"props": {"node.name": 42}}}],
        ):
            with self.subTest(snapshot=snapshot):
                host = GeneratedHost("")
                host.nodes = snapshot
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", 42, PROTOCOL)
                self.assertEqual(host.commands, [JSON_QUERY, QUERY])

    def test_valid_unrelated_objects_do_not_block_named_sink_absence(self):
        host = GeneratedHost("")
        host.nodes = [
            {"id": 10, "type": "PipeWire:Interface:Node",
             "info": {"props": {"node.name": "unrelated-sink"}}},
            {"id": 20, "type": "PipeWire:Interface:Client", "info": {}},
            {"id": 30, "type": "PipeWire:Interface:Metadata"},
        ]
        remove_owned_sink(host, "owned-test", 42, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY])
        self.assertEqual(host.snapshot_count, 1)

    def test_a_lingering_module_or_named_node_uses_one_budget_without_unload_retry(self):
        class PersistentModule(GeneratedHost):
            def text(self, command):
                if command == ["pactl", "unload-module", "42"]:
                    self.commands.append(command)
                    return ""
                return super().text(command)

        node_host = GeneratedHost(FOREIGN)
        node_host.nodes = [{"id": 10, "type": "PipeWire:Interface:Node",
                            "info": {"props": {"node.name": "owned-test"}}}]
        for host, unload_count in [(PersistentModule(), 1), (node_host, 0)]:
            with self.subTest(unload_count=unload_count):
                elapsed = [0.0]
                sleeps = []

                def advance(seconds):
                    sleeps.append(seconds)
                    elapsed[0] += seconds

                with patch("silent_host_modules.time.monotonic", side_effect=lambda: elapsed[0]):
                    with patch("silent_host_modules.time.sleep", side_effect=advance):
                        with self.assertRaisesRegex(EvidenceError, "deadline"):
                            remove_owned_sink(host, "owned-test", 42, PROTOCOL)
                self.assertAlmostEqual(elapsed[0], 5.0)
                self.assertTrue(all(0 < seconds <= 0.1 for seconds in sleeps))
                self.assertEqual(host.commands.count(["pactl", "unload-module", "42"]), unload_count)

    def test_late_absent_snapshot_cannot_pass(self):
        elapsed = [0.0]

        class LateSnapshot(GeneratedHost):
            def snapshot(self):
                result = super().snapshot()
                elapsed[0] = 5.0
                return result

        host = LateSnapshot(FOREIGN)
        with patch("silent_host_modules.time.monotonic", side_effect=lambda: elapsed[0]):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                remove_owned_sink(host, "owned-test", 42, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, QUERY])
        self.assertEqual(host.snapshot_count, 1)


if __name__ == "__main__":
    unittest.main()
