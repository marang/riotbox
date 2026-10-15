"""Generated tempfile controls only; no generator, product runtime or host calls."""

from copy import deepcopy
from dataclasses import replace
from functools import lru_cache
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
import wave

from loaded_recorder_evidence import load_json_bytes, sha256
from loaded_recorder_fixture import BARS, FIXTURE_CONTRACT_SHA256, GeneratedFixture, _compact_json
from silent_host_evidence import EvidenceError


ROOT = Path(__file__).resolve().parents[1]
PRODUCER = "ab" * 32
REJECTIONS = (EvidenceError, OSError)


def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode()


@lru_cache(maxsize=8)
def pcm(rate, frames):
    output = io.BytesIO()
    with wave.open(output, "wb") as audio:
        audio.setframerate(rate)
        audio.setnchannels(2)
        audio.setsampwidth(2)
        audio.writeframes(b"\xe8\x03" * (frames * 2))
    return output.getvalue()


class PreparedControl:
    """Reports/bytes shaped like the real producer, not claimed producer proof."""

    def __init__(self, root):
        self.root = root / "prepared"
        self.root.mkdir()
        for name in ("generated-source", "captures", "offline"):
            (self.root / name).mkdir()
        self.report_path = self.root / "preparation.json"
        self.source_path = self.root / "generated-source/dense_break_132.wav"
        self.capture_path = self.root / "captures/cap-generated.wav"
        self.session_path = self.root / "session.json"
        self.graph_path = self.root / "source-graph.json"
        (self.root / "generator.py").write_bytes(b"# retained generated-only test control\n")
        self.source_path.write_bytes(pcm(44100, 44100 * 32))
        self.capture_path.write_bytes(pcm(44100, 1000))
        source_hash = sha256(self.source_path.read_bytes())
        manifest = {
            "schema": "riotbox.diverse_test_source_wavs.v1", "schema_version": 1,
            "result": "pass", "case_count": 1, "seconds": 32, "sample_rate": 44100, "channels": 2,
            "entries": [{"case_id": "dense_break_132", "source_family": "dense_break", "bpm": 132.0,
                         "expected_role": "break_transients", "path": str(self.source_path),
                         "usage": "generated_test_fixture", "quality_proof": False}],
        }
        (self.root / "generated-source/manifest.json").write_bytes(encode(manifest))
        self.graph = {
            "graph_version": "V1",
            "source": {"source_id": "src-generated", "path": str(self.source_path),
                       "content_hash": "sha256:" + source_hash, "sample_rate": 44100,
                       "channel_count": 2, "duration_seconds": 32.0, "decode_profile": "Native"},
            "timing": {"primary_hypothesis_id": "grid-generated"},
            "provenance": {"source_hash": "sha256:" + source_hash, "analysis_seed": 23},
        }
        self.graph_path.write_bytes(encode(self.graph))
        graph_ref = {
            "source_id": "src-generated", "graph_version": "V1",
            "graph_hash": "sha256:" + sha256(_compact_json(self.graph_path.read_bytes())),
            "storage_mode": "External", "embedded_graph": None,
            "external_path": "source-graph.json", "provenance": self.graph["provenance"],
        }
        capture = {
            "capture_id": "cap-generated", "storage_path": "captures/cap-generated.wav",
            "audio_identity": {"sha256": "sha256:" + sha256(self.capture_path.read_bytes()),
                               "provenance": {"kind": "created_from_encoded_bytes_v1"}},
            "created_from_action": 2, "lineage_capture_refs": [],
            "source_window": {"source_id": "src-generated", "start_seconds": 1.3648977,
                              "end_seconds": 3.1847613, "start_frame": 60192, "end_frame": 140448,
                              "hook_selection": None},
        }
        self.session = {
            "session_version": "V1", "session_id": "session-src-generated",
            "source_refs": [{"source_id": "src-generated", "path_hint": str(self.source_path),
                             "content_hash": "sha256:" + source_hash, "duration_seconds": 32.0,
                             "decode_profile": "native"}],
            "source_graph_refs": [graph_ref], "captures": [capture], "export_receipts": [],
            "runtime_state": {"source_timing": {"confirmed_bpm": 132.0, "confirmed_grid": {
                "source_id": "src-generated", "hypothesis_id": "grid-generated",
                "confirmed_by_action": 1, "confirmed_at": 100}},
                "lane_state": {"w30": {"last_capture": "cap-generated"}}},
            "notes": "literal spaces and escaped \\\"quotes\\\" preserved: café",
        }
        self.session_path.write_bytes(encode(self.session))
        windows = []
        for rate in (44100, 48000):
            for bars in BARS:
                frames = int(rate * 60 * bars * 4 / 132 + 0.5)
                artifact = self.root / f"offline/master-{rate}-{bars}bars.wav"
                artifact.write_bytes(pcm(rate, frames))
                windows.append({
                    "sample_rate_hz": rate, "channel_count": 2, "duration_bars": bars,
                    "frame_count": frames, "requested_start_position_beats": 16.0,
                    "generated_output_geometry": True, "recording_plan_admitted": True,
                    "finite_samples": True, "active_samples": frames * 2,
                    "peak_abs": 1000 / 32768, "rms": 1000 / 32768, "clip_count": 0,
                    "limiter_applied_sample_count": 0, "expected_sample_count": True,
                    "artifact_path": str(artifact), "artifact_sha256": sha256(artifact.read_bytes()),
                    "artifact_error": None, "pattern_origin": "fixture", "passed": True,
                })
        self.report = {
            "schema": "riotbox.generated_live_master_load_preparation.v1", "schema_version": 1,
            "classification": "generated_control_preparation_only", "pattern_origin": "fixture",
            "quality_proof": False, "source_intelligence_qualified": False,
            "host_execution_allowed": False, "live_controller_executed": False,
            "result": "prepared", "failure_reason": None, "contract_sha256": FIXTURE_CONTRACT_SHA256,
            "identity": {
                "generator_repository_path": str(ROOT / "scripts/write_diverse_test_source_wavs.py"),
                "generator_snapshot_path": str(self.root / "generator.py"),
                "generator_sha256": sha256((self.root / "generator.py").read_bytes()),
                "producer_binary_sha256": PRODUCER,
                "generated_source_path": str(self.source_path), "generated_source_sha256": source_hash,
                "session_path": str(self.session_path), "source_graph_path": str(self.graph_path),
                "session_file_sha256": sha256(self.session_path.read_bytes()),
                "source_graph_file_sha256": sha256(self.graph_path.read_bytes()),
                "source_graph_ref": [graph_ref], "capture_artifacts": [capture],
                "core_session_sha256": sha256(_compact_json(self.session_path.read_bytes())),
                "save_restore_identity_verified": True,
                "contributors": ["tr909", "mc202", "w30_preview", "w30_resample_tap", "source_monitor"],
                "contributor_scope": "named composite, not isolated lanes; generated fixture origin",
                "committed_prefix": {"capture_action_id": 2, "raw_audition_action_id": 3,
                                     "promotion_action_id": 4, "preset_action_id": 5,
                                     "preset": "feral_break_alpha_v2", "position_beats": 16.0,
                                     "scripted_fixture_actions": True},
            }, "windows": windows,
        }
        self.write_report()

    def write_report(self):
        self.report_path.write_bytes(encode(self.report))

    def rewrite_session(self):
        data = encode(self.session)
        self.session_path.write_bytes(data)
        identity = self.report["identity"]
        identity["session_file_sha256"] = sha256(data)
        identity["core_session_sha256"] = sha256(_compact_json(data))

    def admit(self, **overrides):
        options = {"expected_report_sha256": sha256(self.report_path.read_bytes()),
                   "expected_producer_sha256": PRODUCER}
        options.update(overrides)
        return GeneratedFixture.admit(self.report_path, **options)


class GeneratedFixtureTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="riotbox-1578-fixture-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.control = PreparedControl(self.root)
        self.output = self.root / "attempt-01"
        self.output.mkdir()

    def test_pinned_fixture_contract_and_valid_admission_without_discovery(self):
        self.assertEqual(sha256((ROOT / "docs/benchmarks/generated_live_master_load_fixture_v1.json").read_bytes()),
                         FIXTURE_CONTRACT_SHA256)
        with patch.object(Path, "glob", side_effect=AssertionError("discovery")), \
                patch.object(Path, "rglob", side_effect=AssertionError("discovery")), \
                patch.object(Path, "iterdir", side_effect=AssertionError("discovery")):
            fixture = self.control.admit()
            fixture.verify_unchanged()
        self.assertEqual(fixture.session_id, self.control.session["session_id"])
        self.assertEqual(fixture.capture_ids, ("cap-generated",))
        self.assertEqual(fixture.source_graph_ref, self.control.session["source_graph_refs"][0])
        self.assertEqual(fixture.report_sha256, sha256(self.control.report_path.read_bytes()))

    def test_three_exact_sibling_sessions_preserve_relative_refs_and_baseline(self):
        fixture = self.control.admit()
        baseline = self.control.session_path.read_bytes()
        for bars in BARS:
            case = fixture.prepare_case(bars, self.output)
            self.assertEqual(case.session_path.parent, self.control.root)
            self.assertEqual(case.session_path.read_bytes(), baseline)
            self.assertEqual(case.destination_path.parent, self.output)
            self.assertEqual(case.proof_path.name, case.destination_path.name + ".riotbox.json")
            self.assertFalse(case.observer_path.exists())
            self.assertEqual(case.session_path.parent / fixture.source_graph_ref["external_path"],
                             self.control.graph_path)
            case.session_path.write_bytes(b"run-local mutation is allowed")
        fixture.verify_unchanged()
        self.assertEqual(self.control.session_path.read_bytes(), baseline)

    def test_public_json_properties_cannot_mutate_admitted_identity(self):
        fixture = self.control.admit()
        for property_name in ("baseline_data", "graph_data", "source_graph_ref", "timing_grid_ref"):
            original = getattr(fixture, property_name)
            changed = getattr(fixture, property_name)
            changed["injected"] = True
            self.assertEqual(getattr(fixture, property_name), original)

    def test_missing_stale_noncanonical_report_or_producer_binding_rejects(self):
        for key in ("expected_report_sha256", "expected_producer_sha256"):
            for value in (None, "cd" * 32, "AB" * 32, "sha256:" + PRODUCER, True):
                with self.subTest(key=key, value=value), self.assertRaises(REJECTIONS):
                    self.control.admit(**{key: value})

    def test_all_generated_claims_protocol_and_result_fail_closed(self):
        original = deepcopy(self.control.report)
        for key, value in (("quality_proof", True), ("quality_proof", 0),
                           ("source_intelligence_qualified", True), ("host_execution_allowed", True),
                           ("live_controller_executed", True), ("result", "failed"),
                           ("pattern_origin", "source_intelligence"), ("schema_version", True),
                           ("contract_sha256", "cd" * 32), ("failure_reason", "retained failure")):
            with self.subTest(key=key, value=value):
                self.control.report = deepcopy(original)
                self.control.report[key] = value
                self.control.write_report()
                with self.assertRaises(REJECTIONS):
                    self.control.admit()

    def test_duplicate_nonfinite_overflow_and_nonobject_json_reject(self):
        for payload in (b'{"schema":1,"schema":2}', b'{"x":NaN}', b'{"x":Infinity}',
                        b'{"x":1e999}', b'[]', b'{} trailing'):
            self.control.report_path.write_bytes(payload)
            with self.subTest(payload=payload), self.assertRaises(REJECTIONS):
                self.control.admit()

    def test_baseline_source_capture_generator_manifest_graph_window_and_report_mutations_reject(self):
        fixture = self.control.admit()
        paths = (self.control.session_path, self.control.source_path, self.control.capture_path,
                 self.control.graph_path, self.control.root / "generator.py",
                 self.control.root / "generated-source/manifest.json", self.control.report_path,
                 *(self.control.root / f"offline/master-{rate}-{bars}bars.wav"
                   for rate in (44100, 48000) for bars in BARS))
        for path in paths:
            original = path.read_bytes()
            path.write_bytes(original + b"mutation")
            with self.subTest(path=path), self.assertRaises(REJECTIONS):
                fixture.verify_unchanged()
            path.write_bytes(original)
        fixture.verify_unchanged()

    def test_missing_or_symlink_baseline_file_rejects(self):
        path = self.control.capture_path
        saved = path.read_bytes()
        path.unlink()
        with self.assertRaises(REJECTIONS):
            self.control.admit()
        target = self.root / "unowned-test-control.wav"
        target.write_bytes(saved)
        path.symlink_to(target)
        with self.assertRaises(REJECTIONS):
            self.control.admit()
        self.assertEqual(target.read_bytes(), saved)

    def test_symlink_owner_ancestor_and_report_reject(self):
        alias = self.root / "alias"
        alias.symlink_to(self.control.root, target_is_directory=True)
        with self.assertRaises(REJECTIONS):
            GeneratedFixture.admit(alias / "preparation.json", expected_report_sha256=PRODUCER,
                                   expected_producer_sha256=PRODUCER)
        original = self.control.report_path.read_bytes()
        target = self.root / "report-target.json"
        target.write_bytes(original)
        self.control.report_path.unlink()
        self.control.report_path.symlink_to(target)
        with self.assertRaises(REJECTIONS):
            self.control.admit()

    def test_escape_paths_reject_before_accessing_external_target(self):
        original = deepcopy(self.control.report)
        for value in (str(self.root / "never-open.wav"), "../never-open.wav", "generated-source/../never-open.wav"):
            self.control.report = deepcopy(original)
            self.control.report["identity"]["generated_source_path"] = value
            self.control.write_report()
            with self.subTest(value=value), self.assertRaises(REJECTIONS):
                self.control.admit()
        self.assertFalse((self.root / "never-open.wav").exists())

    def test_report_capture_and_graph_reference_mismatch_reject(self):
        original = deepcopy(self.control.report)
        for field, value in (("capture_artifacts", []), ("source_graph_ref", []),
                             ("core_session_sha256", PRODUCER), ("session_file_sha256", PRODUCER),
                             ("generated_source_sha256", PRODUCER), ("generator_sha256", None),
                             ("save_restore_identity_verified", False), ("contributors", ["tr909"])):
            self.control.report = deepcopy(original)
            self.control.report["identity"][field] = value
            self.control.write_report()
            with self.subTest(field=field), self.assertRaises(REJECTIONS):
                self.control.admit()

    def test_rebound_session_still_requires_fresh_core_generated_lineage(self):
        original = deepcopy(self.control.session)
        mutations = (
            lambda value: value.update(export_receipts=[{"stale": True}]),
            lambda value: value["source_refs"][0].update(path_hint=str(self.root / "foreign.wav")),
            lambda value: value["captures"][0]["audio_identity"].update(provenance={"kind": "adopted_legacy_v1"}),
            lambda value: value["captures"][0].update(lineage_capture_refs=["unknown"]),
            lambda value: value["runtime_state"]["lane_state"]["w30"].update(last_capture="unknown"),
            lambda value: value["runtime_state"]["source_timing"].update(confirmed_bpm=133),
        )
        for index, mutate in enumerate(mutations):
            self.control.session = deepcopy(original)
            mutate(self.control.session)
            self.control.rewrite_session()
            # Rebind duplicated report references so these tests cross the
            # Session identity checks, rather than just duplicated-value checks.
            self.control.report["identity"]["capture_artifacts"] = deepcopy(self.control.session["captures"])
            self.control.write_report()
            with self.subTest(index=index), self.assertRaises(REJECTIONS):
                self.control.admit()

    def test_all_six_windows_required_in_exact_order_and_hash_bound(self):
        original = deepcopy(self.control.report)
        variants = (lambda windows: windows.pop(), lambda windows: windows.reverse(),
                    lambda windows: windows[0].update(artifact_sha256=None),
                    lambda windows: windows[0].update(artifact_sha256=PRODUCER),
                    lambda windows: windows[0].update(frame_count=1),
                    lambda windows: windows[0].update(passed=False),
                    lambda windows: windows[0].update(peak_abs=1.0),
                    lambda windows: windows[0].update(clip_count=1),
                    lambda windows: windows[0].update(active_samples=0),
                    lambda windows: windows[0].update(artifact_error="failed"),
                    lambda windows: windows[0].update(channel_count=True))
        for index, mutate in enumerate(variants):
            self.control.report = deepcopy(original)
            mutate(self.control.report["windows"])
            self.control.write_report()
            with self.subTest(index=index), self.assertRaises(REJECTIONS):
                self.control.admit()

    def test_geometry_readback_rejects_hash_rebound_truncated_window(self):
        window = self.control.report["windows"][0]
        path = Path(window["artifact_path"])
        payload = path.read_bytes()[:-4]
        path.write_bytes(payload)
        window["artifact_sha256"] = sha256(payload)
        self.control.write_report()
        with self.assertRaises(REJECTIONS):
            self.control.admit()

    def test_capture_report_promoted_f32_numbers_match_bits_not_decimal_text(self):
        captures = deepcopy(self.control.session["captures"])
        window = captures[0]["source_window"]
        window.update(start_seconds=1.3648977279663086, end_seconds=3.1847612857818604)
        self.control.report["identity"]["capture_artifacts"] = captures
        self.control.write_report()
        self.control.admit()
        bits = struct.unpack("<I", struct.pack("<f", window["start_seconds"]))[0]
        window["start_seconds"] = struct.unpack("<f", struct.pack("<I", bits + 1))[0]
        self.control.write_report()
        with self.assertRaises(REJECTIONS):
            self.control.admit()

    def test_typed_hook_selection_f32_fields_match_but_other_fields_stay_exact(self):
        short, promoted = 1.3648977, 1.3648977279663086
        hook = {"policy": "transport_boundary_v1", "reason": "transport_boundary_policy",
                "baseline_bar_index": 1, "baseline_start_seconds": short, "baseline_end_seconds": short,
                "selected_bar_index": 1, "baseline_score": short, "selected_score": short, "score_lift": short,
                "selected_evidence": {name: short for name in ("onset_flux", "body_retention", "onset_density",
                                    "spectral_contrast", "repetition_salience", "phrase_contrast")}}
        self.control.session["captures"][0]["source_window"]["hook_selection"] = hook
        self.control.rewrite_session()
        captures = deepcopy(self.control.session["captures"])
        reported_hook = captures[0]["source_window"]["hook_selection"]
        for key in ("baseline_start_seconds", "baseline_end_seconds", "baseline_score", "selected_score", "score_lift"):
            reported_hook[key] = promoted
        for key in reported_hook["selected_evidence"]:
            reported_hook["selected_evidence"][key] = promoted
        self.control.report["identity"]["capture_artifacts"] = captures
        self.control.write_report()
        self.control.admit()
        for key, value in (("selected_bar_index", 1.0), ("policy", "changed"), ("score_lift", True)):
            saved = reported_hook[key]
            reported_hook[key] = value
            self.control.write_report()
            with self.subTest(key=key), self.assertRaises(REJECTIONS):
                self.control.admit()
            reported_hook[key] = saved

    def test_only_actual_core_session_native_graph_Native_wire_profiles_are_supported(self):
        self.control.admit()
        for session_profile, graph_profile in (("Native", "Native"), ("native", "native"),
                                                ("Native", "native"), ("custom", "Custom"),
                                                ({"Custom": "native"}, {"Custom": "native"})):
            self.control.session["source_refs"][0]["decode_profile"] = session_profile
            self.control.graph["source"]["decode_profile"] = graph_profile
            graph_bytes = encode(self.control.graph)
            self.control.graph_path.write_bytes(graph_bytes)
            self.control.session["source_graph_refs"][0]["graph_hash"] = "sha256:" + sha256(_compact_json(graph_bytes))
            self.control.report["identity"]["source_graph_file_sha256"] = sha256(graph_bytes)
            self.control.report["identity"]["source_graph_ref"] = deepcopy(self.control.session["source_graph_refs"])
            self.control.rewrite_session()
            self.control.write_report()
            with self.subTest(session=session_profile, graph=graph_profile), self.assertRaises(REJECTIONS):
                self.control.admit()

    def test_invalid_duration_and_repeat_allocation_reject(self):
        fixture = self.control.admit()
        for bars in (True, False, 2.0, "2", 4, 0, 32):
            with self.subTest(bars=bars), self.assertRaises(REJECTIONS):
                fixture.prepare_case(bars, self.output)
        case = fixture.prepare_case(2, self.output)
        case.session_path.unlink()  # Test-owned allocation; no reuse even if removed.
        with self.assertRaises(REJECTIONS):
            fixture.prepare_case(2, self.output)

    def test_every_occupied_file_or_symlink_rejects_before_session_copy(self):
        fixture = self.control.admit()
        session = self.control.root / "run-attempt-01-2bars.session.json"
        candidates = (session, self.output / "master-2bars.wav",
                      self.output / "master-2bars.wav.riotbox.json", self.output / "observer-2bars.ndjson")
        for path in candidates:
            for symlink in (False, True):
                target = self.output / "keep-target"
                if symlink:
                    target.write_bytes(b"keep")
                    path.symlink_to(target)
                else:
                    path.write_bytes(b"keep")
                with self.subTest(path=path, symlink=symlink), self.assertRaises(REJECTIONS):
                    fixture.prepare_case(2, self.output)
                self.assertEqual(path.read_bytes(), b"keep")
                if path != session:
                    self.assertFalse(session.exists())
                path.unlink()
                if symlink:
                    self.assertEqual(target.read_bytes(), b"keep")
                    target.unlink()

    def test_output_owner_must_exist_be_separate_and_have_no_links(self):
        fixture = self.control.admit()
        alias = self.root / "alias-output"
        alias.symlink_to(self.output, target_is_directory=True)
        nested = self.control.root / "nested"
        nested.mkdir()
        for owner in (self.root / "missing", alias, self.control.root, nested, Path("relative")):
            with self.subTest(owner=owner), self.assertRaises(REJECTIONS):
                fixture.prepare_case(2, owner)

    def test_changed_baseline_prevents_case_copy(self):
        fixture = self.control.admit()
        self.control.capture_path.write_bytes(b"changed")
        with self.assertRaises(REJECTIONS):
            fixture.prepare_case(2, self.output)
        self.assertFalse((self.control.root / "run-attempt-01-2bars.session.json").exists())

    def test_prelaunch_rejects_sequential_source_redirection_before_reading_it(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        redirected = deepcopy(self.control.session)
        redirected["source_refs"][0]["path_hint"] = str(self.root / "foreign-do-not-read.wav")
        case.session_path.write_bytes(encode(redirected))
        with self.assertRaisesRegex(EvidenceError, "copy changed before launch"):
            fixture.verify_case_before_launch(case)
        fixture.verify_unchanged()
        self.assertFalse((self.root / "foreign-do-not-read.wav").exists())

    def test_prelaunch_rejects_unregistered_changed_or_mistyped_case_before_any_read(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        foreign = self.root / "foreign-never-read.json"
        variants = [replace(case, **{key: foreign}) for key in
                    ("session_path", "destination_path", "proof_path", "observer_path")]
        variants += [replace(case, bars=8), replace(case, bars=2.0),
                     replace(case, session_path=str(case.session_path)), object()]
        for changed in variants:
            with self.subTest(case=changed), patch("loaded_recorder_fixture._read",
                    side_effect=AssertionError("unregistered path reached file reader")), \
                    self.assertRaises(EvidenceError):
                fixture.verify_case_before_launch(changed)
        fixture.verify_case_before_launch(case)

    def test_prelaunch_checks_outputs_again_and_rejects_new_files_and_links(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        for path in (case.destination_path, case.proof_path, case.observer_path):
            for symlink in (False, True):
                if symlink:
                    path.symlink_to(self.output / "missing-target")
                else:
                    path.write_bytes(b"occupied after allocation")
                with self.subTest(path=path, symlink=symlink), self.assertRaises(EvidenceError):
                    fixture.verify_case_before_launch(case)
                self.assertTrue(path.is_symlink() or path.read_bytes() == b"occupied after allocation")
                path.unlink()  # Generated test-owned collision only.
        fixture.verify_case_before_launch(case)

    def test_prelaunch_consumes_valid_case_once_even_before_any_recording(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        fixture.verify_case_before_launch(case)
        with patch("loaded_recorder_fixture._read", side_effect=AssertionError("consumed case read")), \
                self.assertRaisesRegex(EvidenceError, "already consumed"):
            fixture.verify_case_before_launch(case)

    def test_prelaunch_rejects_already_recorded_copy_missing_or_symlinked_session(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        baseline = case.session_path.read_bytes()
        recorded = deepcopy(self.control.session)
        recorded["export_receipts"] = [{"receipt_id": "already-recorded"}]
        case.session_path.write_bytes(encode(recorded))
        with self.assertRaisesRegex(EvidenceError, "copy changed before launch"):
            fixture.verify_case_before_launch(case)
        case.session_path.unlink()
        with self.assertRaises(REJECTIONS):
            fixture.verify_case_before_launch(case)
        target = self.output / "run-session-alias-target.json"
        target.write_bytes(baseline)
        case.session_path.symlink_to(target)
        with self.assertRaisesRegex(EvidenceError, "symlink"):
            fixture.verify_case_before_launch(case)

    def test_prelaunch_rechecks_baseline_and_output_owner_ancestors(self):
        fixture = self.control.admit()
        case = fixture.prepare_case(2, self.output)
        baseline = self.control.session_path.read_bytes()
        self.control.session_path.write_bytes(b"mutated baseline")
        with self.assertRaisesRegex(EvidenceError, "baseline changed"):
            fixture.verify_case_before_launch(case)
        self.control.session_path.write_bytes(baseline)
        relocated = self.root / "moved-owned-output"
        self.output.rename(relocated)
        self.output.symlink_to(relocated, target_is_directory=True)
        with self.assertRaisesRegex(EvidenceError, "symlink"):
            fixture.verify_case_before_launch(case)

    def test_whitespace_compaction_preserves_rust_numbers_and_escaped_string_bytes(self):
        pretty = b'{\n "f32": 1e-6, "integer": 1, "s": " x \\\" y \\\\ z ", "n": -0.0\n}\n'
        expected = b'{"f32":1e-6,"integer":1,"s":" x \\\" y \\\\ z ","n":-0.0}'
        self.assertEqual(load_json_bytes(pretty), load_json_bytes(expected))
        self.assertEqual(_compact_json(pretty), expected)


if __name__ == "__main__":
    unittest.main()
