"""Generated paired module metadata; never queries a real host."""

import unittest
from unittest.mock import patch

import silent_host_modules as modules
from silent_host_evidence import EvidenceError


JSON_QUERY = ["pactl", "--format=json", "list", "short", "modules"]
TEXT_QUERY = ["pactl", "--format=text", "list", "short", "modules"]
OWNER = "owned-v3-test"
ARGUMENT = f"sink_name={OWNER} channels=2 channel_map=front-left,front-right"
PROTOCOL = {"teardown_deadline_seconds": 5, "route_poll_interval_ms": 100}


def record(index, name="module-null-sink", argument=ARGUMENT):
    return {"index": index, "name": name, "argument": argument}


def paired(records, *, trailing_tab=True, normalized=False):
    payloads = [{"name": row["name"], "argument": row["argument"]} for row in records]
    text = "".join(
        f'{row["index"]}\t{row["name"]}\t'
        + row["argument"].replace("\r\n", "\n").replace("\r", "\n")
        + ("\t\n" if trailing_tab else "\n") for row in records
    )
    return GeneratedHost(payloads, text.rstrip("\r\n") if normalized else text)


class GeneratedHost:
    def __init__(self, payloads, text, nodes=None):
        self.payloads, self.module_text = payloads, text
        self.nodes = [] if nodes is None else nodes
        self.commands = []

    def json(self, command):
        self.commands.append(command)
        if command != JSON_QUERY:
            raise AssertionError(command)
        return self.payloads

    def text(self, command):
        self.commands.append(command)
        if command == TEXT_QUERY:
            return self.module_text
        if command == ["pactl", "unload-module", "42"]:
            self.payloads, self.module_text = [], ""
            return ""
        raise AssertionError(command)

    def snapshot(self):
        self.commands.append(["generated-snapshot"])
        return self.nodes


class ModuleFramingV3Tests(unittest.TestCase):
    def test_native_multiline_tabs_apostrophe_fake_header_are_opaque(self):
        native = record(7, "libpipewire-module-rt", "{\n# listener's comment\n\tx = 'unterminated\n"
                        f"42\tmodule-null-sink\t{ARGUMENT}\t\n}}")
        host = paired([native, record(42)])
        self.assertEqual(modules._read_modules(host), [native, record(42)])
        self.assertEqual(modules.owned_modules(paired([native, record(42)]), OWNER), [record(42)])
        self.assertEqual(host.commands, [JSON_QUERY, TEXT_QUERY])

    def test_newline_projection_preserves_original_identity(self):
        native = record(7, "libpipewire-module-rt", "{\r\n\tx=1\r# comment\n}\r\n")
        for trailing_tab in (False, True):
            for normalized in (False, True):
                with self.subTest(trailing_tab=trailing_tab, normalized=normalized):
                    self.assertEqual(modules._read_modules(paired(
                        [native], trailing_tab=trailing_tab, normalized=normalized)), [native])

    def test_json_order_is_not_index_authority_and_identical_payloads_keep_counts(self):
        rows = [record(7, "module-always-sink", ""), record(42),
                record(9, "module-always-sink", "")]
        host = paired(rows)
        host.payloads.reverse()
        host.payloads[0]["index"] = 999
        self.assertEqual(modules._read_modules(host), rows)

    def test_both_column_shapes_final_normalization_and_empty_table(self):
        for trailing_tab in (False, True):
            for normalized in (False, True):
                with self.subTest(trailing_tab=trailing_tab, normalized=normalized):
                    self.assertEqual(modules._read_modules(paired(
                        [record(0, "module-always-sink", ""), record(4294967294)],
                        trailing_tab=trailing_tab, normalized=normalized)),
                        [record(0, "module-always-sink", ""), record(4294967294)])
        self.assertEqual(modules._read_modules(paired([])), [])

    def test_competing_payload_prefix_rejects_without_backtracking(self):
        rows = [record(7, "module-a", "x"), record(8, "module-a", "x\t\n9\tmodule-b\ty")]
        host = paired(rows)
        host.module_text = "7\tmodule-a\tx\t\n9\tmodule-b\ty\t\n"
        with self.assertRaisesRegex(EvidenceError, "ambiguous"):
            modules._read_modules(host)

    def test_distinct_crlf_projections_are_ambiguous(self):
        for rows in ([record(7, "module-a", "x\r\ny"), record(8, "module-a", "x\ny")],
                     [record(7, "module-a", "x\ry"), record(8, "module-a", "x\ny")]):
            with self.subTest(rows=rows), self.assertRaisesRegex(EvidenceError, "ambiguous"):
                modules._read_modules(paired(rows))

    def test_terminal_normalization_ambiguity_rejects_and_spaces_are_not_trimmed(self):
        rows = [record(7, "module-a", "x"), record(8, "module-a", "x\n")]
        host = paired(rows, trailing_tab=False, normalized=True)
        host.module_text = "7\tmodule-a\tx"
        with self.assertRaisesRegex(EvidenceError, "ambiguous"):
            modules._read_modules(host)
        row = record(7, "libpipewire-module-rt", "\t { value = 1 } \t ")
        self.assertEqual(modules._read_modules(paired([row], normalized=True)), [row])

    def test_stale_partial_extra_or_changed_payload_rejects(self):
        for change in ("missing-json", "extra-json", "changed-json", "missing-text", "extra-text"):
            host = paired([record(42)])
            if change == "missing-json":
                host.payloads = []
            elif change == "extra-json":
                host.payloads.append({"name": "module-a", "argument": ""})
            elif change == "changed-json":
                host.payloads[0]["argument"] += " rate=1"
            elif change == "missing-text":
                host.module_text = ""
            else:
                host.module_text += "7\tmodule-a\t\t\n"
            with self.subTest(change=change), self.assertRaises(EvidenceError):
                modules._read_modules(host)
            self.assertNotIn(["pactl", "unload-module", "42"], host.commands)

    def test_malformed_json_and_control_names_reject(self):
        payloads = (None, {}, "[]", [None], [{"name": "module-a"}],
                    [{"name": "", "argument": ""}], [{"name": 1, "argument": ""}],
                    [{"name": "module-a", "argument": None}],
                    [{"name": "module-a", "argument": []}])
        for payload in payloads:
            with self.subTest(payload=payload), self.assertRaises(EvidenceError):
                modules._read_modules(GeneratedHost(payload, ""))
        for control in ("\t", "\n", "\r", "\x00", "\x7f", "\x85"):
            with self.subTest(control=control), self.assertRaises(EvidenceError):
                modules._read_modules(paired([record(7, "module" + control, "")]))

    def test_canonical_unique_indices_and_malformed_text_reject(self):
        for index in ("", "042", "+42", "-1", "42.0", "0x2a", "４２", "4294967295",
                      "4294967296", "9" * 30):
            host = paired([record(index)])
            with self.subTest(index=index), self.assertRaises(EvidenceError):
                modules._read_modules(host)
        for host in (paired([record(42), record(42)]), GeneratedHost([], None),
                     GeneratedHost([], []), GeneratedHost([], "\n")):
            with self.subTest(host=host), self.assertRaises(EvidenceError):
                modules._read_modules(host)
        for suffix in ("\tgarbage\n", "\t\t\n", "\n\n"):
            host = paired([record(42)])
            host.module_text = host.module_text[:-2] + suffix
            with self.subTest(suffix=suffix), self.assertRaises(EvidenceError):
                modules._read_modules(host)

    def test_native_owner_like_body_is_not_owned_but_bound_id_is_replacement(self):
        native = record(42, "libpipewire-module-loopback", ARGUMENT + " # author's comment")
        self.assertEqual(modules.owned_modules(paired([native]), OWNER), [])
        host = paired([native])
        with self.assertRaisesRegex(EvidenceError, "replacement"):
            modules.remove_owned_sink(host, OWNER, 42, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY, TEXT_QUERY])

    def test_strict_pulse_ownership_rejects_wrong_type_conflicts_and_malformed(self):
        for row in (record(42, "module-remap-sink"), record(42, argument="'" + ARGUMENT),
                    record(42, argument=ARGUMENT + " sink_name=other"),
                    record(42, argument=ARGUMENT + " sink_name=" + OWNER),
                    record(42, argument=ARGUMENT + " channels=2"),
                    record(42, argument=ARGUMENT.replace("channels=2", "channels=1"))):
            with self.subTest(row=row), self.assertRaises(EvidenceError):
                modules.owned_modules(paired([row]), OWNER)
        with self.assertRaises(EvidenceError):
            modules.owned_modules(paired([record(42), record(43)]), OWNER)
        for argument in (f'sink_properties="a sink_name={OWNER} b"',
                         ARGUMENT.replace(OWNER, OWNER + "-other")):
            self.assertEqual(modules.owned_modules(paired([record(42, argument=argument)]), OWNER), [])
        quoted = record(42, argument=f"channel_map=front-left,front-right sink_name='{OWNER}' channels=2")
        self.assertEqual(modules.owned_modules(paired([quoted]), OWNER), [quoted])

    def test_cleanup_unique_owner_without_ack_unloads_only_real_index(self):
        host = paired([record(42)])
        modules.remove_owned_sink(host, OWNER, None, PROTOCOL)
        self.assertEqual(host.commands.count(["pactl", "unload-module", "42"]), 1)

    def test_native_replacement_after_unload_cannot_establish_absence(self):
        class ReplacedHost(GeneratedHost):
            def text(self, command):
                result = super().text(command)
                if command == ["pactl", "unload-module", "42"]:
                    replacement = paired([record(42, "libpipewire-module-rt", "# author's body")])
                    self.payloads, self.module_text = replacement.payloads, replacement.module_text
                return result

        initial = paired([record(42)])
        host = ReplacedHost(initial.payloads, initial.module_text)
        with self.assertRaisesRegex(EvidenceError, "replacement"):
            modules.remove_owned_sink(host, OWNER, 42, PROTOCOL)
        self.assertEqual(host.commands.count(["pactl", "unload-module", "42"]), 1)
        self.assertNotIn(["generated-snapshot"], host.commands)

    def test_deadlines_before_queries_between_queries_and_during_parsing(self):
        for expires_after, expected_commands in ((0, []), (1, [JSON_QUERY]),
                                                 (3, [JSON_QUERY, TEXT_QUERY]),
                                                 (6, [JSON_QUERY, TEXT_QUERY]),
                                                 (9, [JSON_QUERY, TEXT_QUERY])):
            host = paired([record(42)])
            calls = 0

            def clock():
                nonlocal calls
                calls += 1
                return 0 if calls <= expires_after else 2

            with self.subTest(expires_after=expires_after), patch.object(modules.time, "monotonic", clock):
                with self.assertRaisesRegex(EvidenceError, "deadline"):
                    modules.owned_modules(host, OWNER, deadline=1)
                self.assertEqual(host.commands, expected_commands)

    def test_cleanup_shared_deadline_forbids_unload_after_late_pair(self):
        host = paired([record(42)])
        calls = 0

        def clock():
            nonlocal calls
            calls += 1
            return 0 if calls < 5 else 6

        with patch.object(modules.time, "monotonic", clock):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                modules.remove_owned_sink(host, OWNER, 42, PROTOCOL)
        self.assertNotIn(["pactl", "unload-module", "42"], host.commands)

    def test_named_node_absence_requires_validated_identity(self):
        modules.require_sink_absent(GeneratedHost([], ""), OWNER)
        for node in ({"id": 7, "type": "PipeWire:Interface:Node", "info": {"props": {"node.name": OWNER}}},
                     {"id": 7, "type": "PipeWire:Interface:Node", "info": {"props": {}}},
                     {"id": 7, "type": None}):
            host = GeneratedHost([], "", [node])
            with self.subTest(node=node), self.assertRaises(EvidenceError):
                modules.require_sink_absent(host, OWNER)
            self.assertEqual(host.commands, [["generated-snapshot"]])


if __name__ == "__main__":
    unittest.main()
