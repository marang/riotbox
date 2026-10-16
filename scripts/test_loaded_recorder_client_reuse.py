"""V2 lifetime regressions through real supervision, generated controls only."""

from contextlib import contextmanager
import unittest

import test_loaded_recorder_process as controls
from loaded_recorder_evidence import LoadedObservationVersion
from silent_host_evidence import EvidenceError
from silent_host_routes import RouteError


@contextmanager
def generated_case():
    # Reuse the existing process/clock/metadata adapters without inheriting or
    # rerunning their test methods. Route, lifetime and cleanup remain real.
    case = controls.ProcessTests()
    try:
        case.setUp()
        case.observation_version = LoadedObservationVersion.V2
        case.scenario.recycled_client = True
        yield case
    finally:
        case.doCleanups()


def mutate_client(case, mutate):
    original = case.host.snapshot

    def snapshot():
        data = original()
        for item in data:
            if item["id"] == 30 and item["type"] == "PipeWire:Interface:Client":
                mutate(item)
        return data

    case.host.snapshot = snapshot


class ClientReuseTests(unittest.TestCase):
    def test_decimal_serial_forms_and_client_name_are_not_special_cased(self):
        for serial in (4000, "4000", 2**64 - 1, str(2**64 - 1)):
            with self.subTest(serial=serial), generated_case() as case:
                mutate_client(case, lambda item: item["info"]["props"].update({
                    "object.serial": serial, "application.name": "unrelated generated client"}))
                self.assertTrue(case.run_case()["result"]["ready"])
                self.assertTrue(case.process.cleanup_verified)

    def test_missing_invalid_overflow_and_original_serial_are_not_absence(self):
        for serial in (None, True, False, -1, 0, 2**64, 3000, "-1", "invalid", [], {}):
            with self.subTest(serial=serial), generated_case() as case:
                mutate_client(case, lambda item: item["info"]["props"].update({"object.serial": serial}))
                with self.assertRaises((EvidenceError, RouteError)):
                    case.run_case()
                self.assertTrue(case.process.cleanup_verified)
                self.assertEqual(case.diagnostics.finished, [(42, False)])
                self.assertLess(case.clock.value, 101 * controls.NS)

    def test_only_client_interface_qualifies_not_replaced_nodes_or_prefixes(self):
        for interface in ("Node", "Port", "Link", "Metadata", "Unknown", "", None, 1):
            with self.subTest(interface=interface), generated_case() as case:
                replacement = (f"PipeWire:Interface:{interface}"
                               if isinstance(interface, str) else interface)
                mutate_client(case, lambda item: item.update(type=replacement))
                with self.assertRaises((EvidenceError, RouteError)):
                    case.run_case()
                self.assertTrue(case.process.cleanup_verified)
                self.assertEqual(case.diagnostics.finished, [(42, False)])

    def test_client_info_and_properties_must_remain_maps(self):
        for field in ("info", "props"):
            with self.subTest(field=field), generated_case() as case:
                def corrupt(item):
                    if field == "info":
                        item["info"] = []
                    else:
                        item["info"]["props"] = []
                mutate_client(case, corrupt)
                with self.assertRaises(RouteError):
                    case.run_case()
                self.assertTrue(case.process.cleanup_verified)

    def test_delayed_marker_resolves_client_absence_without_extending_budget(self):
        with generated_case() as case:
            case.scenario.visible, case.scenario.exit_at = 102.0, 102.2
            result = case.run_case()
            self.assertTrue(result["result"]["ready"])
            self.assertLess(case.clock.value, 103 * controls.NS)

    def test_client_absence_still_rejects_premature_completion(self):
        with generated_case() as case:
            case.scenario.complete, case.scenario.stopped = 100.5, 100.6
            case.scenario.visible, case.scenario.exit_at = 100.7, 101.0
            with self.assertRaisesRegex(EvidenceError, "before window completion"):
                case.run_case()
            self.assertTrue(case.process.cleanup_verified)
            self.assertEqual(case.diagnostics.finished, [(42, False)])

    def test_v2_cannot_admit_overlap_or_malformed_marker(self):
        for invalid in ("overlap", "malformed", "wrong PID", "duplicate"):
            with self.subTest(invalid=invalid), generated_case() as case:
                if invalid == "overlap":
                    case.scenario.complete, case.scenario.stopped = 100.115, 100.116
                else:
                    case.scenario.visible = 100.22
                    case.scenario.bad_marker = invalid
                with self.assertRaises(EvidenceError):
                    case.run_case()
                self.assertTrue(case.process.cleanup_verified)

    def test_missing_marker_and_hung_finalization_keep_the_original_deadline(self):
        for visible in (None, 100.12):
            with self.subTest(visible=visible), generated_case() as case:
                case.scenario.visible, case.scenario.exit_at = visible, None
                with self.assertRaises(controls.operator.UnverifiedCleanup) as caught:
                    case.run_case()
                details = str(caught.exception) + " ".join(getattr(caught.exception, "__notes__", []))
                self.assertIn("deadline", details)
                self.assertLess(case.clock.value, 106 * controls.NS)

    def test_client_reuse_does_not_defer_foreign_link_or_default_change(self):
        for unsafe in ("foreign_call", "changed_default"):
            with self.subTest(unsafe=unsafe), generated_case() as case:
                if unsafe == "foreign_call":
                    case.scenario.foreign_call = 3
                else:
                    original = case.host.default_state
                    case.host.default_state = lambda: ({"unsafe": True} if case.host.calls >= 3
                                                       else original())
                with self.assertRaises((EvidenceError, RouteError)):
                    case.run_case()
                self.assertLess(case.clock.value, 101 * controls.NS)

    def test_original_route_cannot_reappear_after_client_absence(self):
        with generated_case() as case:
            original = case.host.snapshot
            def snapshot():
                if case.process.active and case.host.calls == 3:
                    case.scenario.absence_call = 100
                return original()
            case.host.snapshot = snapshot
            with self.assertRaisesRegex(EvidenceError, "reappeared"):
                case.run_case()

    def test_client_reuse_in_verified_teardown_is_not_a_surviving_audio_node(self):
        with generated_case() as case:
            original = case.host.snapshot
            def snapshot():
                data = original()
                if not case.process.active:
                    data.append({"id": 30, "type": "PipeWire:Interface:Client",
                                 "info": {"props": {"object.serial": 4000,
                                                    "application.process.id": 999}}})
                return data
            case.host.snapshot = snapshot
            self.assertTrue(case.run_case()["result"]["ready"])
            self.assertTrue(case.process.cleanup_verified)

    def test_non_typed_version_rejects_before_any_fixture_or_process_work(self):
        with generated_case() as case:
            case.observation_version = "v2"
            with self.assertRaisesRegex(EvidenceError, "version"):
                case.run_case()
            self.assertEqual(case.fixture.calls, 0)
            self.assertIsNone(case.process)


if __name__ == "__main__":
    unittest.main()
