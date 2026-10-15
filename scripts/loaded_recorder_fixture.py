"""Admission of one explicitly bound generated V1 owner; never discovers inputs.

This module does not generate, hydrate or play audio and grants no execution
permission. The caller supplies an already exclusively owned output directory.
Only named files from the prepared owner are read; run Sessions are exact sibling
copies so ordinary Session-relative references retain their meaning.
"""

from copy import deepcopy
from dataclasses import dataclass
import io
import math
import os
from pathlib import Path
import re
import stat
import struct
import wave

from loaded_recorder_evidence import load_json_bytes, require, sha256
from silent_host_evidence import EvidenceError, integer


FIXTURE_CONTRACT_SHA256 = "18981581fe9e762c48fc120f0b325a9172b49194ac2cb686f5865eed0fe0b08d"
BARS = (2, 8, 16)
RATES = (44100, 48000)
_HASH = re.compile(r"[0-9a-f]{64}\Z")
_OWNER_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,95}\Z")
_MAX_FILE_BYTES = 64 * 1024 * 1024
_UNBOUND = object()


def _require(condition, reason):
    if not condition:
        raise EvidenceError(reason)


def _object(value, name):
    _require(type(value) is dict, f"{name} must be an object")
    return value


def _text(value, name):
    _require(type(value) is str and bool(value.strip()), f"invalid {name}")
    return value


def _hash(value, name, *, prefixed=False):
    _text(value, name)
    if prefixed:
        _require(value.startswith("sha256:"), f"missing SHA256 prefix: {name}")
        value = value[7:]
    _require(_HASH.fullmatch(value) is not None, f"noncanonical SHA256: {name}")
    return value


def _content_hash(value, name):
    return _hash(value, name, prefixed=isinstance(value, str) and value.startswith("sha256:"))


def _absolute(path):
    value = os.fspath(path)
    result = Path(value)
    _require(result.is_absolute() and str(result) == value
             and ".." not in result.parts, "path must be explicit absolute canonical spelling")
    return result


def _no_links(path, *, directory=False):
    """Reject links in every existing path segment, not just at the leaf."""
    path = _absolute(path)
    current = Path(path.anchor)
    for segment in path.parts[1:]:
        current /= segment
        metadata = current.lstat()
        _require(not stat.S_ISLNK(metadata.st_mode), f"symlink is not an owned path: {current}")
        if current != path or directory:
            _require(stat.S_ISDIR(metadata.st_mode), f"directory required: {current}")
    return path


def _inside(owner, value):
    _text(value, "declared fixture path")
    raw = Path(value)
    _require(str(raw) == value and ".." not in raw.parts, "noncanonical fixture path")
    path = raw if raw.is_absolute() else owner / raw
    _require(path != owner and path.is_relative_to(owner), "fixture path escapes generated owner")
    return path


def _read(path):
    _no_links(path)
    # NONBLOCK avoids hanging on a FIFO; fstat rejects everything but a regular
    # file before reading. NOFOLLOW also closes the ordinary leaf-link race.
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        _require(stat.S_ISREG(metadata.st_mode), "fixture input is not a regular file")
        _require(metadata.st_size <= _MAX_FILE_BYTES, "fixture input exceeds bounded file budget")
        data = stream.read(_MAX_FILE_BYTES + 1)
    _require(len(data) <= _MAX_FILE_BYTES, "fixture input grew past bounded file budget")
    return data


def _compact_json(data):
    """Preserve serde numeric/string spellings while removing pretty whitespace.

    Used only AFTER strict JSON admission. Python reserialization would change
    some f32/exponent spellings and cannot reproduce Core's serialized identity.
    """
    result = bytearray()
    quoted = escaped = False
    for byte in data:
        if quoted or byte not in b" \t\r\n":
            result.append(byte)
        if quoted:
            if escaped:
                escaped = False
            elif byte == 92:
                escaped = True
            elif byte == 34:
                quoted = False
        elif byte == 34:
            quoted = True
    return bytes(result)


def _capture_identity(value):
    """Canonicalize ONLY Core-declared CaptureRef f32 leaves to exact bits.

    The Session serializer writes shortest f32 decimals, while json!(captures)
    in the report first promotes those same f32s into Value's f64 numbers.
    This is typed equality, not a tolerance or general float normalization.
    Field names follow CaptureSourceWindow, W30HookSelectionDecision and
    source_graph::W30HookFeatures; every other leaf retains strict wire equality.
    """
    capture = deepcopy(_object(value, "capture reference"))

    def fields(parent, names, *, optional=False):
        for key in names:
            number = parent.get(key)
            if optional and number is None:
                continue
            _require(type(number) in (int, float) and math.isfinite(number), f"invalid capture f32: {key}")
            try:
                bits = struct.pack("<f", number)
            except (OverflowError, struct.error) as error:
                raise EvidenceError(f"out-of-range capture f32: {key}") from error
            _require(math.isfinite(struct.unpack("<f", bits)[0]), f"nonfinite capture f32: {key}")
            parent[key] = bits

    window = capture.get("source_window")
    if window is not None:
        window = _object(window, "capture source window")
        fields(window, ("start_seconds", "end_seconds"))
        selection = window.get("hook_selection")
        if selection is not None:
            selection = _object(selection, "capture hook selection")
            fields(selection, ("baseline_start_seconds", "baseline_end_seconds"))
            fields(selection, ("baseline_score", "selected_score", "score_lift"), optional=True)
            evidence = selection.get("selected_evidence")
            if evidence is not None:
                evidence = _object(evidence, "capture hook features")
                fields(evidence, ("onset_flux", "body_retention", "onset_density",
                                  "spectral_contrast", "repetition_salience", "phrase_contrast"))
    return capture


def _pcm16_geometry(data, *, rate=None, channels=None, frames=None):
    try:
        with wave.open(io.BytesIO(data), "rb") as audio:
            actual = (audio.getframerate(), audio.getnchannels(), audio.getnframes())
            _require(audio.getsampwidth() == 2 and audio.getcomptype() == "NONE",
                     "generated artifact is not PCM16")
            _require(len(audio.readframes(actual[2])) == actual[1] * actual[2] * 2,
                     "generated artifact payload is truncated")
    except (wave.Error, EOFError) as error:
        raise EvidenceError("malformed generated WAV") from error
    expected = (rate, channels, frames)
    _require(all(wanted is None or wanted == got for wanted, got in zip(expected, actual)),
             "generated artifact geometry mismatch")
    _require(all(value > 0 for value in actual), "empty generated artifact geometry")


def _absent(path):
    _no_links(path.parent, directory=True)
    try:
        path.lstat()
    except FileNotFoundError:
        return
    raise EvidenceError(f"occupied recording output: {path}")


@dataclass(frozen=True)
class RecordingCase:
    bars: int
    session_path: Path
    destination_path: Path
    proof_path: Path
    observer_path: Path


class GeneratedFixture:
    """Validated immutable inputs plus exclusive mutable Session allocation."""

    @classmethod
    def admit(cls, report_path, *, expected_report_sha256, expected_producer_sha256):
        fixture = cls()
        fixture.report_sha256 = _hash(expected_report_sha256, "preparation report")
        _hash(expected_producer_sha256, "preparation producer")
        fixture.owner = _no_links(_absolute(report_path).parent, directory=True)
        report_path = _absolute(report_path)
        _require(report_path == fixture.owner / "preparation.json", "unexpected preparation report name")
        fixture._identities = {}
        fixture._prepared = set()
        fixture._cases = {}
        fixture._consumed = set()
        data = fixture._remember(report_path, expected_report_sha256)
        report = load_json_bytes(data)
        fixture._admit_report(report, expected_producer_sha256)
        fixture.verify_unchanged()
        return fixture

    def _remember(self, path, expected_hash=_UNBOUND):
        path = _inside(self.owner, str(path))
        data = _read(path)
        digest = sha256(data)
        if expected_hash is not _UNBOUND:
            _require(digest == _hash(expected_hash, str(path)), "generated fixture byte identity mismatch")
        if path in self._identities:
            _require(self._identities[path] == digest, "generated path has conflicting identities")
        self._identities[path] = digest
        return data

    def _admit_report(self, report, producer_hash):
        _require(report.get("schema") == "riotbox.generated_live_master_load_preparation.v1"
                 and type(report.get("schema_version")) is int and report["schema_version"] == 1
                 and report.get("contract_sha256") == FIXTURE_CONTRACT_SHA256
                 and report.get("classification") == "generated_control_preparation_only"
                 and report.get("pattern_origin") == "fixture"
                 and report.get("result") == "prepared" and report.get("failure_reason") is None,
                 "unsupported or unsuccessful generated preparation")
        for flag in ("quality_proof", "source_intelligence_qualified", "host_execution_allowed",
                     "live_controller_executed"):
            _require(report.get(flag) is False, f"generated preparation made unsupported claim: {flag}")
        identity = _object(report.get("identity"), "preparation identity")
        _require(identity.get("producer_binary_sha256") == _hash(producer_hash, "producer"),
                 "generated producer binding mismatch")
        _require(identity.get("save_restore_identity_verified") is True, "restore identity unverified")
        _require(identity.get("contributors") == ["tr909", "mc202", "w30_preview",
                 "w30_resample_tap", "source_monitor"], "generated composite identity changed")
        _require(identity.get("contributor_scope") ==
                 "named composite, not isolated lanes; generated fixture origin", "composite scope changed")
        generator = _inside(self.owner, identity.get("generator_snapshot_path"))
        _require(generator == self.owner / "generator.py", "unexpected generator snapshot")
        # repository_path is provenance only: never follow/read it outside owner.
        _text(identity.get("generator_repository_path"), "generator repository provenance")
        self._remember(generator, identity.get("generator_sha256"))
        source_path = _inside(self.owner, identity.get("generated_source_path"))
        _require(source_path == self.owner / "generated-source/dense_break_132.wav", "unexpected generated input")
        source_hash = _hash(identity.get("generated_source_sha256"), "generated source")
        _pcm16_geometry(self._remember(source_path, source_hash), rate=44100, channels=2, frames=32 * 44100)
        self._admit_manifest(source_path)
        self.baseline_session = _inside(self.owner, identity.get("session_path"))
        graph_path = _inside(self.owner, identity.get("source_graph_path"))
        _require(self.baseline_session == self.owner / "session.json"
                 and graph_path == self.owner / "source-graph.json", "unexpected baseline paths")
        self._session_bytes = self._remember(self.baseline_session, identity.get("session_file_sha256"))
        self._baseline_data = load_json_bytes(self._session_bytes)
        _require(sha256(_compact_json(self._session_bytes)) ==
                 _hash(identity.get("core_session_sha256"), "Core Session"), "Core Session identity mismatch")
        graph_bytes = self._remember(graph_path, identity.get("source_graph_file_sha256"))
        self._graph_data = load_json_bytes(graph_bytes)
        self._admit_session(identity, source_path, source_hash, graph_path, graph_bytes)
        self._admit_windows(report.get("windows"))

    def _admit_manifest(self, source_path):
        manifest = load_json_bytes(self._remember(self.owner / "generated-source/manifest.json"))
        _require(manifest.get("schema") == "riotbox.diverse_test_source_wavs.v1"
                 and type(manifest.get("schema_version")) is int and manifest["schema_version"] == 1
                 and manifest.get("result") == "pass"
                 and integer(manifest.get("case_count"), "source count") == 1
                 and manifest.get("seconds") == 32 and manifest.get("sample_rate") == 44100
                 and manifest.get("channels") == 2, "generated manifest geometry/origin mismatch")
        entries = manifest.get("entries")
        _require(type(entries) is list and len(entries) == 1, "unexpected generated manifest entries")
        entry = _object(entries[0], "generated entry")
        _require(entry.get("case_id") == "dense_break_132" and entry.get("source_family") == "dense_break"
                 and entry.get("bpm") == 132 and entry.get("expected_role") == "break_transients"
                 and entry.get("usage") == "generated_test_fixture" and entry.get("quality_proof") is False
                 and _inside(self.owner, entry.get("path")) == source_path, "generated manifest entry mismatch")

    def _admit_session(self, identity, source_path, source_hash, graph_path, graph_bytes):
        session, graph = self._baseline_data, self._graph_data
        _require(session.get("session_version") == "V1" and session.get("export_receipts") == [],
                 "baseline is not a fresh unrecorded V1 Session")
        self.session_id = _text(session.get("session_id"), "Session ID")
        refs = session.get("source_graph_refs")
        _require(type(refs) is list and len(refs) == 1 and refs == identity.get("source_graph_ref"),
                 "baseline/report Graph references disagree")
        self._source_graph_ref = _object(refs[0], "Graph reference")
        ref = self._source_graph_ref
        source = _object(graph.get("source"), "Graph source")
        source_id = _text(source.get("source_id"), "source ID")
        _require(graph.get("graph_version") == "V1" and ref.get("graph_version") == "V1"
                 and ref.get("source_id") == source_id
                 and ref.get("storage_mode") == "External" and ref.get("embedded_graph") is None
                 and _inside(self.owner, ref.get("external_path")) == graph_path
                 and _hash(ref.get("graph_hash"), "Core Graph", prefixed=True) == sha256(_compact_json(graph_bytes))
                 and ref.get("provenance") == graph.get("provenance"), "Core Graph lineage mismatch")
        _require(_inside(self.owner, source.get("path")) == source_path
                 and _content_hash(source.get("content_hash"), "Graph source") == source_hash
                 and source.get("sample_rate") == 44100 and source.get("channel_count") == 2
                 and source.get("duration_seconds") == 32
                 and source.get("decode_profile") == "Native"
                 and _content_hash(_object(graph.get("provenance"), "Graph provenance").get("source_hash"),
                                   "Graph provenance") == source_hash, "Graph generated-source identity mismatch")
        source_refs = session.get("source_refs")
        _require(type(source_refs) is list and len(source_refs) == 1, "unexpected baseline sources")
        src = _object(source_refs[0], "Session source")
        _require(src.get("source_id") == source_id and _inside(self.owner, src.get("path_hint")) == source_path
                 and _content_hash(src.get("content_hash"), "Session source") == source_hash
                 and src.get("duration_seconds") == 32
                 # Core's Session adapter intentionally serializes this V1
                 # profile differently from SourceGraph's derived enum wire.
                 and src.get("decode_profile") == "native", "Session generated-source mismatch")
        runtime = _object(session.get("runtime_state"), "baseline runtime")
        timing = _object(runtime.get("source_timing"), "baseline source timing")
        self._timing_grid_ref = _object(timing.get("confirmed_grid"), "confirmed timing grid")
        _require(timing.get("confirmed_bpm") == 132 and self._timing_grid_ref.get("source_id") == source_id
                 and self._timing_grid_ref.get("hypothesis_id") ==
                 _object(graph.get("timing"), "Graph timing").get("primary_hypothesis_id"), "baseline timing identity mismatch")
        integer(self._timing_grid_ref.get("confirmed_by_action"), "timing action")
        integer(self._timing_grid_ref.get("confirmed_at"), "timing timestamp")
        captures = session.get("captures")
        reported_captures = identity.get("capture_artifacts")
        _require(type(captures) is list and bool(captures) and type(reported_captures) is list,
                 "baseline/report capture identities required")
        require([_capture_identity(capture) for capture in captures],
                [_capture_identity(capture) for capture in reported_captures], "baseline/report capture identities")
        capture_ids = []
        capture_paths = set()
        for capture in captures:
            capture = _object(capture, "baseline capture")
            capture_ids.append(_text(capture.get("capture_id"), "capture ID"))
            path = _inside(self.owner, capture.get("storage_path"))
            _require(path.is_relative_to(self.owner / "captures") and path not in capture_paths,
                     "capture path is duplicate or outside declared capture directory")
            capture_paths.add(path)
            audio_identity = _object(capture.get("audio_identity"), "capture audio identity")
            _require(audio_identity.get("provenance") == {"kind": "created_from_encoded_bytes_v1"},
                     "capture lacks generated encoded-byte provenance")
            capture_hash = _hash(audio_identity.get("sha256"), "capture", prefixed=True)
            _pcm16_geometry(self._remember(path, capture_hash))
        _require(len(set(capture_ids)) == len(capture_ids), "duplicate capture ID")
        for capture in captures:
            lineage = capture.get("lineage_capture_refs")
            _require(type(lineage) is list and all(ref in capture_ids for ref in lineage), "unknown capture lineage")
        w30 = _object(_object(runtime.get("lane_state"), "baseline lanes").get("w30"), "baseline W-30")
        _require(w30.get("last_capture") in capture_ids, "baseline active capture is undeclared")
        self.capture_ids = tuple(capture_ids)
        prefix = _object(identity.get("committed_prefix"), "committed prefix")
        _require(prefix.get("scripted_fixture_actions") is True and prefix.get("preset") == "feral_break_alpha_v2",
                 "generated committed prefix changed")
        for key in ("capture_action_id", "raw_audition_action_id", "promotion_action_id", "preset_action_id"):
            _require(integer(prefix.get(key), key) > 0, "missing committed prefix action")
        _require(any(capture.get("created_from_action") == prefix["capture_action_id"] for capture in captures),
                 "capture does not belong to committed prefix")

    def _admit_windows(self, windows):
        _require(type(windows) is list and len(windows) == 6, "exactly six prepared windows required")
        for window, (rate, bars) in zip(windows, ((rate, bars) for rate in RATES for bars in BARS)):
            window = _object(window, "prepared window")
            frames = math.floor(rate * 60 * bars * 4 / 132 + 0.5)
            _require(integer(window.get("sample_rate_hz"), "window rate") == rate
                     and integer(window.get("duration_bars"), "window bars") == bars
                     and integer(window.get("channel_count"), "window channels") == 2
                     and integer(window.get("frame_count"), "window frames") == frames,
                     "prepared window geometry/order mismatch")
            for flag in ("generated_output_geometry", "recording_plan_admitted", "finite_samples",
                         "expected_sample_count", "passed"):
                _require(window.get(flag) is True, f"window preparation did not pass: {flag}")
            peak, rms, start = (window.get(key) for key in ("peak_abs", "rms", "requested_start_position_beats"))
            _require(all(type(value) in (int, float) and math.isfinite(value) for value in (peak, rms, start))
                     and 0 < peak < 1 and rms >= 0 and start >= 0
                     and 0 < integer(window.get("active_samples"), "active samples") <= frames * 2
                     and integer(window.get("clip_count"), "clip count") == 0
                     and integer(window.get("limiter_applied_sample_count"), "limited samples") <= frames * 2
                     and window.get("artifact_error") is None and window.get("pattern_origin") == "fixture",
                     "prepared window technical evidence mismatch")
            path = _inside(self.owner, window.get("artifact_path"))
            _require(path == self.owner / f"offline/master-{rate}-{bars}bars.wav", "unexpected window artifact")
            _pcm16_geometry(self._remember(path, window.get("artifact_sha256")), rate=rate, channels=2, frames=frames)

    @property
    def baseline_data(self):
        return deepcopy(self._baseline_data)

    @property
    def graph_data(self):
        return deepcopy(self._graph_data)

    @property
    def source_graph_ref(self):
        return deepcopy(self._source_graph_ref)

    @property
    def timing_grid_ref(self):
        return deepcopy(self._timing_grid_ref)

    def verify_unchanged(self):
        for path, digest in self._identities.items():
            _require(sha256(_read(path)) == digest, f"prepared baseline changed: {path}")

    def prepare_case(self, bars, output_owner):
        _require(type(bars) is int and bars in BARS and bars not in self._prepared,
                 "unsupported or already allocated loaded case")
        output_owner = _no_links(_absolute(output_owner), directory=True)
        _require(output_owner != self.owner and not output_owner.is_relative_to(self.owner),
                 "recording output owner must be separate from immutable preparation")
        _require(_OWNER_NAME.fullmatch(output_owner.name) is not None, "unsafe recording owner name")
        session_path = self.owner / f"run-{output_owner.name}-{bars}bars.session.json"
        destination = output_owner / f"master-{bars}bars.wav"
        case = RecordingCase(bars, session_path, destination,
                             destination.with_name(destination.name + ".riotbox.json"),
                             output_owner / f"observer-{bars}bars.ndjson")
        self.verify_unchanged()
        # Check every declared destination BEFORE producing any mutable copy.
        for path in (case.session_path, case.destination_path, case.proof_path, case.observer_path):
            _absent(path)
        fd = os.open(session_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        self._prepared.add(bars)
        with os.fdopen(fd, "wb") as stream:
            stream.write(self._session_bytes)
            stream.flush()
        _require(_read(session_path) == self._session_bytes, "run Session copy changed")
        self.verify_unchanged()
        self._cases[bars] = case
        return case

    def verify_case_before_launch(self, case):
        """Re-admit the exact allocated copy immediately before its one launch.

        Baseline preservation alone cannot protect ingress through a mutable
        run Session. Reject sequential edits BEFORE the CLI may hydrate them.
        This does not promise protection against a concurrent same-user writer.
        """
        _require(type(case) is RecordingCase and type(case.bars) is int,
                 "loaded launch requires an exact registered RecordingCase")
        registered = self._cases.get(case.bars)
        _require(registered is not None and case.bars not in self._consumed,
                 "loaded case is unregistered or already consumed")
        # Do not inspect/open any caller-supplied path until every field agrees
        # with the successfully allocated case, including concrete path types.
        for key in ("bars", "session_path", "destination_path", "proof_path", "observer_path"):
            actual, expected = getattr(case, key), getattr(registered, key)
            _require(type(actual) is type(expected) and actual == expected,
                     "loaded case identity changed before launch")
        self.verify_unchanged()
        _require(_read(registered.session_path) == self._session_bytes,
                 "run Session copy changed before launch")
        for path in (registered.destination_path, registered.proof_path, registered.observer_path):
            _absent(path)
        self.verify_unchanged()
        self._consumed.add(registered.bars)
