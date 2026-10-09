"""Independent generated metadata at the public module-observation seam.

No subprocess, operator invocation, host metadata, or local audio is used.
"""

import unittest
from unittest.mock import patch

from silent_host_evidence import EvidenceError
from silent_host_modules import owned_modules, remove_owned_sink


JSON_QUERY = ["pactl", "--format=json", "list", "short", "modules"]
TEXT_QUERY = ["pactl", "--format=text", "list", "short", "modules"]
OWNER_ARGUMENT = "sink_name=owned-test channels=2 channel_map=front-left,front-right"
OWNER_PAYLOAD = {"name": "module-null-sink", "argument": OWNER_ARGUMENT}
OWNER_TEXT = "42\tmodule-null-sink\tsink_name=owned-test channels=2 channel_map=front-left,front-right\t\n"
PROTOCOL = {"teardown_deadline_seconds": 5, "route_poll_interval_ms": 100}


class GeneratedMetadataHost:
    """Predeclared metadata responses; unexpected commands cannot reach a host."""

    def __init__(self, payloads, text, *, unloaded=None, on_query=None):
        self.payloads = payloads
        self.module_text = text
        self.unloaded = unloaded
        self.on_query = on_query
        self.commands = []
        self.nodes = []

    def _observe(self, command):
        self.commands.append(command)
        if self.on_query is not None:
            self.on_query(command)

    def json(self, command):
        if command != JSON_QUERY:
            raise AssertionError(f"unexpected generated JSON command: {command}")
        self._observe(command)
        return self.payloads

    def text(self, command):
        self._observe(command)
        if command == TEXT_QUERY:
            return self.module_text
        if command == ["pactl", "unload-module", "42"] and self.unloaded is not None:
            self.payloads, self.module_text = self.unloaded
            return ""
        raise AssertionError(f"unexpected generated text command: {command}")

    def snapshot(self):
        self.commands.append(["pw-dump"])
        return self.nodes

    def unloads(self):
        return [command for command in self.commands if command[:2] == ["pactl", "unload-module"]]


class NativeCompatibilityTests(unittest.TestCase):
    def test_native_multiline_tab_body_coexists_with_exact_owned_module(self):
        payload = {"name": "libpipewire-module-rt", "argument": "{\n\tnice.level = -11\n}"}
        host = GeneratedMetadataHost(
            [OWNER_PAYLOAD, payload],
            "7\tlibpipewire-module-rt\t{\n\tnice.level = -11\n}\t\n" + OWNER_TEXT,
        )
        self.assertEqual(owned_modules(host, "owned-test"), [
            {"index": 42, "name": "module-null-sink", "argument": OWNER_ARGUMENT},
        ])

    def test_native_comment_apostrophe_is_not_a_malformed_pulse_argument(self):
        payload = {"name": "libpipewire-module-rt", "argument": "# musician's priority"}
        host = GeneratedMetadataHost(
            [payload, OWNER_PAYLOAD],
            "7\tlibpipewire-module-rt\t# musician's priority\t\n" + OWNER_TEXT,
        )
        self.assertEqual(owned_modules(host, "owned-test")[0]["index"], 42)


class OwnershipSafetyTests(unittest.TestCase):
    def test_native_fake_header_cannot_manufacture_owner_or_unload(self):
        payload = {
            "name": "libpipewire-module-rt",
            "argument": "x\t\n42\tmodule-null-sink\tsink_name=owned-test channels=2 channel_map=front-left,front-right",
        }
        text = "7\tlibpipewire-module-rt\tx\t\n42\tmodule-null-sink\tsink_name=owned-test channels=2 channel_map=front-left,front-right\t\n"
        host = GeneratedMetadataHost([payload], text)
        self.assertEqual(owned_modules(host, "owned-test"), [])
        remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])

    def test_native_owner_tuple_is_opaque_even_without_comments_or_braces(self):
        host = GeneratedMetadataHost(
            [{"name": "libpipewire-module-rt", "argument": OWNER_ARGUMENT}],
            "7\tlibpipewire-module-rt\tsink_name=owned-test channels=2 channel_map=front-left,front-right\t\n",
        )
        self.assertEqual(owned_modules(host, "owned-test"), [])
        remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])

    def test_quoted_property_contents_are_not_top_level_ownership(self):
        host = GeneratedMetadataHost(
            [{"name": "module-null-sink", "argument": 'sink_properties="a sink_name=owned-test b"'}],
            '42\tmodule-null-sink\tsink_properties="a sink_name=owned-test b"\t\n',
        )
        self.assertEqual(owned_modules(host, "owned-test"), [])
        remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])

    def test_real_text_index_not_json_position_or_extra_index_owns_unload(self):
        foreign = {"name": "libpipewire-module-rt", "argument": "# musician's priority", "index": 42}
        host = GeneratedMetadataHost(
            [{**OWNER_PAYLOAD, "index": 900}, foreign],
            "7\tlibpipewire-module-rt\t# musician's priority\t\n" + OWNER_TEXT,
            unloaded=([foreign], "7\tlibpipewire-module-rt\t# musician's priority\t\n"),
        )
        self.assertEqual(owned_modules(host, "owned-test"), [
            {"index": 42, "name": "module-null-sink", "argument": OWNER_ARGUMENT},
        ])
        remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [["pactl", "unload-module", "42"]])
        self.assertEqual(host.payloads, [foreign])

    def test_native_record_at_bound_id_is_replacement_not_absence(self):
        for body in ("# musician's priority", OWNER_ARGUMENT, '{\n\tname = "owned-test"\n}'):
            with self.subTest(body=body):
                host = GeneratedMetadataHost(
                    [{"name": "libpipewire-module-rt", "argument": body}],
                    "42\tlibpipewire-module-rt\t" + body + "\t\n",
                )
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", 42, PROTOCOL)
                self.assertEqual(host.unloads(), [])

    def test_duplicate_owned_payloads_at_distinct_indices_never_select_an_owner(self):
        host = GeneratedMetadataHost(
            [OWNER_PAYLOAD, OWNER_PAYLOAD],
            OWNER_TEXT + "43\tmodule-null-sink\tsink_name=owned-test channels=2 channel_map=front-left,front-right\t\n",
        )
        with self.assertRaises(EvidenceError):
            remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])


class PairedFramingTests(unittest.TestCase):
    def test_text_newline_projection_preserves_original_identity_argument(self):
        argument = "sink_name=owned-test\r\nchannels=2\rchannel_map=front-left,front-right"
        host = GeneratedMetadataHost(
            [{"name": "module-null-sink", "argument": argument}],
            "42\tmodule-null-sink\tsink_name=owned-test\nchannels=2\nchannel_map=front-left,front-right\t",
        )
        self.assertEqual(owned_modules(host, "owned-test"), [
            {"index": 42, "name": "module-null-sink", "argument": argument},
        ])

    def test_argument_trailing_whitespace_is_not_trimmed_into_a_record_terminator(self):
        argument = "sink_name=owned-test channels=2 channel_map=front-left,front-right \t\n"
        host = GeneratedMetadataHost(
            [{"name": "module-null-sink", "argument": argument}],
            "42\tmodule-null-sink\tsink_name=owned-test channels=2 channel_map=front-left,front-right \t\n\t",
        )
        self.assertEqual(owned_modules(host, "owned-test"), [
            {"index": 42, "name": "module-null-sink", "argument": argument},
        ])

    def test_identical_foreign_payload_multiplicity_and_empty_arguments_are_valid(self):
        foreign = {"name": "module-always-sink", "argument": ""}
        host = GeneratedMetadataHost(
            [foreign, OWNER_PAYLOAD, foreign],
            "7\tmodule-always-sink\t\t\n8\tmodule-always-sink\t\t\n" + OWNER_TEXT,
        )
        self.assertEqual(owned_modules(host, "owned-test")[0]["index"], 42)

    def test_competing_prefix_payloads_fail_closed_without_backtracking(self):
        host = GeneratedMetadataHost(
            [
                {"name": "libpipewire-module-rt", "argument": "short"},
                {"name": "libpipewire-module-rt", "argument": "short\t\n8\tlibpipewire-module-rt\tshort"},
            ],
            "7\tlibpipewire-module-rt\tshort\t\n8\tlibpipewire-module-rt\tshort\t\n9\tlibpipewire-module-rt\tshort\t\n",
        )
        with self.assertRaises(EvidenceError):
            remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])

    def test_distinct_original_arguments_with_same_newline_projection_are_ambiguous(self):
        host = GeneratedMetadataHost(
            [
                {"name": "libpipewire-module-rt", "argument": "a\r\nb"},
                {"name": "libpipewire-module-rt", "argument": "a\nb"},
            ],
            "7\tlibpipewire-module-rt\ta\nb\t\n8\tlibpipewire-module-rt\ta\nb\t\n",
        )
        with self.assertRaises(EvidenceError):
            remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])

    def test_stale_partial_and_unconsumed_tables_cannot_authorize_unload(self):
        foreign = {"name": "module-always-sink", "argument": ""}
        cases = [
            ([OWNER_PAYLOAD], OWNER_TEXT.replace("owned-test", "other")),
            ([OWNER_PAYLOAD], ""),
            ([], OWNER_TEXT),
            ([OWNER_PAYLOAD, foreign], OWNER_TEXT),
            ([OWNER_PAYLOAD], OWNER_TEXT + "7\tmodule-always-sink\t\t\n"),
            ([OWNER_PAYLOAD], OWNER_TEXT[:-15]),
            ([OWNER_PAYLOAD], OWNER_TEXT + "\n"),
            ([OWNER_PAYLOAD, foreign], OWNER_TEXT + "42\tmodule-always-sink\t\t\n"),
        ]
        for payloads, text in cases:
            with self.subTest(payloads=payloads, text=text):
                host = GeneratedMetadataHost(payloads, text)
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", None, PROTOCOL)
                self.assertEqual(host.unloads(), [])

    def test_malformed_json_fields_never_authorize_an_owned_text_record(self):
        for payloads in (
            {}, None, "not an array", [None],
            [{"name": "module-null-sink"}],
            [{"name": "module-null-sink", "argument": None}],
            [{"name": "module-null-sink", "argument": 42}],
            [{"name": "", "argument": OWNER_ARGUMENT}],
            [{"name": "module-null-sink\n", "argument": OWNER_ARGUMENT}],
            [{"name": "module-null-sink\t", "argument": OWNER_ARGUMENT}],
            [{"name": "module-null-sink\x00", "argument": OWNER_ARGUMENT}],
        ):
            with self.subTest(payloads=payloads):
                host = GeneratedMetadataHost(payloads, OWNER_TEXT)
                with self.assertRaises(EvidenceError):
                    remove_owned_sink(host, "owned-test", None, PROTOCOL)
                self.assertEqual(host.unloads(), [])


class SharedDeadlineTests(unittest.TestCase):
    def test_json_query_consumes_same_cleanup_deadline_before_text_or_unload(self):
        clock = [0.0]

        def query_finished(command):
            if command == JSON_QUERY:
                clock[0] = 5.0

        host = GeneratedMetadataHost([OWNER_PAYLOAD], OWNER_TEXT, on_query=query_finished)
        with patch("silent_host_modules.time.monotonic", side_effect=lambda: clock[0]):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.commands, [JSON_QUERY])
        self.assertEqual(host.unloads(), [])

    def test_late_text_response_never_authorizes_unload(self):
        clock = [0.0]

        def query_finished(command):
            if command == TEXT_QUERY:
                clock[0] = 5.0

        host = GeneratedMetadataHost([OWNER_PAYLOAD], OWNER_TEXT, on_query=query_finished)
        with patch("silent_host_modules.time.monotonic", side_effect=lambda: clock[0]):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])
        self.assertNotIn(["pw-dump"], host.commands)

    def test_pure_framing_cooperates_with_the_callers_deadline(self):
        host, monotonic = self._expiring_large_table()
        with patch("silent_host_modules.time.monotonic", side_effect=monotonic):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                owned_modules(host, "owned-test", deadline=5.0)
        self.assertEqual(host.unloads(), [])
        self.assertNotIn(["pw-dump"], host.commands)

    def test_cleanup_cannot_unload_after_budget_expires_during_framing(self):
        host, monotonic = self._expiring_large_table()
        with patch("silent_host_modules.time.monotonic", side_effect=monotonic):
            with self.assertRaisesRegex(EvidenceError, "deadline"):
                remove_owned_sink(host, "owned-test", None, PROTOCOL)
        self.assertEqual(host.unloads(), [])
        self.assertNotIn(["pw-dump"], host.commands)

    @staticmethod
    def _expiring_large_table():
        # Clock stays still through both external queries, then progresses
        # during CPU-only work. No parser internals are mocked or inspected.
        framing = [False]
        clock = [0.0]

        def query_finished(command):
            if command == TEXT_QUERY:
                framing[0] = True
                clock[0] = 4.5

        def monotonic():
            if framing[0]:
                clock[0] += 0.05
            return clock[0]

        foreign = {"name": "libpipewire-module-rt", "argument": ""}
        text = "".join(f"{index}\tlibpipewire-module-rt\t\t\n" for index in range(100, 700))
        host = GeneratedMetadataHost(
            [foreign] * 600 + [OWNER_PAYLOAD], text + OWNER_TEXT, on_query=query_finished,
        )
        return host, monotonic


if __name__ == "__main__":
    unittest.main()
