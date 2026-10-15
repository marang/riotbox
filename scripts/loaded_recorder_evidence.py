"""Closed, source-free loaded-recorder evidence contract; not execution consent."""

import hashlib
import json
import math
from pathlib import Path
import re

from silent_host_evidence import EvidenceError, integer, load_protocol_v3
from silent_host_diagnostics import load_protocol_v4

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_SHA256 = "687bea31dfa903a299fc14b9e01567bf34b3cd4536220397f2cbb6f780e943ca"
FIXTURE_SHA256 = "18981581fe9e762c48fc120f0b325a9172b49194ac2cb686f5865eed0fe0b08d"
MILESTONE = "live_master_recording_window_stopped"
LIMIT = 262144


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def require(value, expected, name):
    def equal(left, right):
        if type(left) is not type(right):
            return False
        if isinstance(left, dict):
            return left.keys() == right.keys() and all(equal(left[key], right[key]) for key in left)
        if isinstance(left, list):
            return len(left) == len(right) and all(equal(a, b) for a, b in zip(left, right))
        return left == right

    if not equal(value, expected):
        raise EvidenceError(f"loaded recorder {name} mismatch")


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError(f"duplicate JSON field: {key}")
        result[key] = value
    return result


def load_json_bytes(data):
    def reject_constant(value):
        raise EvidenceError(f"non-finite JSON constant: {value}")

    try:
        value = json.loads(data, object_pairs_hook=_pairs, parse_constant=reject_constant)
    except (ValueError, UnicodeError, RecursionError) as error:
        raise EvidenceError("invalid loaded recorder JSON") from error
    if not isinstance(value, dict):
        raise EvidenceError("loaded recorder JSON must be an object")
    # JSON exponent overflow is not handled by parse_constant.
    def finite(item):
        if isinstance(item, float) and not math.isfinite(item):
            raise EvidenceError("non-finite loaded recorder JSON number")
        if isinstance(item, dict):
            for child in item.values():
                finite(child)
        elif isinstance(item, list):
            for child in item:
                finite(child)
    try:
        finite(value)
    except RecursionError as error:
        raise EvidenceError("loaded recorder JSON nesting exceeds parser capacity") from error
    return value


def load_protocol(root=ROOT):
    data = (root / "docs/benchmarks/loaded_recorder_observation_v1.json").read_bytes()
    require(sha256(data), PROTOCOL_SHA256, "protocol bytes")
    protocol = load_json_bytes(data)
    previous = load_protocol_v3(root / "docs/benchmarks/silent_host_observation_v3.json")
    diagnostic = load_protocol_v4(root)
    for key in ("process_timeout_seconds", "kill_grace_seconds", "metadata_timeout_seconds",
                "route_poll_interval_ms", "startup_route_deadline_seconds",
                "teardown_deadline_seconds", "preflight_process_timeout_seconds"):
        require(protocol[key], previous[key], key)
        require(protocol[key], diagnostic[key], key)
    fixture = (root / "docs/benchmarks/generated_live_master_load_fixture_v1.json").read_bytes()
    require(sha256(fixture), protocol["generated_fixture_contract_sha256"], "fixture contract")
    return protocol


def _hex(value, name):
    if not isinstance(value, str) or re.fullmatch("[0-9a-f]{64}", value) is None:
        raise EvidenceError(f"invalid SHA256: {name}")


def validate_marker(marker, pid, case, fixture):
    for key, expected in {
        "event": MILESTONE, "schema": "riotbox.user_session_observer.v1",
        "observation_schema": "riotbox.live_master_recording_window_stopped.v1",
        "pid": pid, "clock_domain": "linux_clock_monotonic", "opt_in": True,
        "capture_context": "non_interactive_real_audio_cli", "realtime_callback_io": False,
        "raw_audio_recording": False, "live_master_recording": True,
        "window_complete": True, "runtime_stopped": True, "fault_count": 0,
        "session_id": fixture.session_id, "duration_bars": case.bars,
        "duration_beats": case.bars * 4, "channel_count": 2,
    }.items():
        require(marker.get(key), expected, key)
    complete = integer(marker.get("window_complete_monotonic_ns"), "window complete time")
    stopped = integer(marker.get("runtime_stopped_monotonic_ns"), "runtime stopped time")
    if not 0 < complete <= stopped:
        raise EvidenceError("loaded recorder milestone clock is unavailable or reversed")
    for key in ("ready", "status", "success"):
        if key in marker:
            raise EvidenceError("loaded recorder milestone cannot carry a verdict")
    for key in ("frame_count", "sample_rate_hz", "action_id"):
        if integer(marker.get(key), key) == 0:
            raise EvidenceError(f"zero loaded recorder {key}")
    if marker["sample_rate_hz"] not in (44100, 48000):
        raise EvidenceError("loaded recorder rate is outside generated fixture geometry")
    _hex(marker.get("session_pre_capture_sha256"), "pre-capture Session")
    start = marker.get("requested_start_position_beats")
    # The confirmed Source Graph may phase bars at beat 3, not beat 0. The
    # marker has no bar-anchor field: final proof + Core validate that phase.
    if type(start) not in (float, int) or not math.isfinite(start) or start < 0:
        raise EvidenceError("loaded recorder requested start is invalid")
    graph = {key: fixture.source_graph_ref[key]
             for key in ("source_id", "graph_version", "graph_hash")}
    require(marker.get("source_graph_ref"), graph, "source graph reference")
    require(marker.get("timing_grid_ref"), fixture.timing_grid_ref, "timing reference")
    baseline = fixture.baseline_data
    active = baseline["runtime_state"]["lane_state"]["w30"]["last_capture"]
    capture = next((item for item in baseline["captures"] if item["capture_id"] == active), None)
    if capture is None:
        raise EvidenceError("loaded fixture has no active capture")
    expected_refs = {"source_capture_refs": [active],
                     "lineage_capture_refs": sorted(set(capture["lineage_capture_refs"]))}
    for key in ("source_capture_refs", "lineage_capture_refs"):
        refs = marker.get(key)
        if (not isinstance(refs, list) or any(not isinstance(ref, str) for ref in refs)
                or len(set(refs)) != len(refs)
                or not set(refs).issubset(fixture.capture_ids)):
            raise EvidenceError(f"loaded recorder {key} is not bound to the fixture")
        require(refs, expected_refs[key], key)
    previous = baseline["action_log"]["actions"]
    if marker["action_id"] <= max((action["id"] for action in previous), default=0):
        raise EvidenceError("loaded recorder action is not fresh")
    return marker


def read_observer(path, pid, case, fixture, *, complete=False):
    if path.is_symlink():
        raise EvidenceError("loaded observer path is a symlink")
    try:
        with path.open("rb") as source:
            data = source.read(LIMIT + 1)
    except FileNotFoundError:
        if complete:
            raise EvidenceError("loaded observer is missing") from None
        return None
    if len(data) > LIMIT or (complete and data and not data.endswith(b"\n")):
        raise EvidenceError("loaded observer exceeds budget or ends with incomplete record")
    lines = data.split(b"\n")[:-1]
    if len(lines) > 2:
        raise EvidenceError("loaded observer has duplicate or unexpected records")
    records = [load_json_bytes(line) for line in lines]
    if not records:
        if complete:
            raise EvidenceError("loaded observer has no stopped milestone")
        return None
    marker = validate_marker(records[0], pid, case, fixture)
    if len(records) == 2:
        terminal = records[1]
        for key, expected in {
            "event": "live_master_recording_execute", "schema": "riotbox.user_session_observer.v1",
            "opt_in": True, "capture_context": "non_interactive_real_audio_cli",
            "realtime_callback_io": False, "raw_audio_recording": False,
            "live_master_recording": True,
        }.items():
            require(terminal.get(key), expected, "terminal " + key)
        summary = terminal.get("summary", {})
        require(summary.get("ready"), True, "terminal ready")
        require(summary.get("runtime_stopped"), True, "terminal stopped")
        require(summary.get("session_path"), str(case.session_path), "terminal Session")
        if summary.get("observer_status") == "write_failed":
            raise EvidenceError("loaded observer retained a milestone write failure")
    elif complete:
        raise EvidenceError("loaded observer has no terminal record")
    return marker


def validate_final(summary, marker, case, fixture):
    # Separate semantic owner; local import avoids exposing a second product model.
    from loaded_recorder_artifacts import validate_final as validate_artifacts
    data = case.observer_path.read_bytes()
    terminal = load_json_bytes(data.splitlines()[-1])["summary"]
    for key in ("observer_events", "observer_status", "observer_failure_reason"):
        terminal.pop(key, None)
    stdout = {key: value for key, value in summary.items()
              if key not in ("observer_events", "observer_status", "observer_failure_reason")}
    require(terminal, stdout, "terminal observer/stdout")
    return validate_artifacts(summary, marker, case, fixture)


def verify_loaded_geometry(diagnostics, measurement, summary):
    # Explicit geometry attribution, not a fabricated silent-driver transcript.
    if diagnostics is None:
        raise EvidenceError("loaded recorder requires bound ALSA diagnostics")
    active = measurement["active_pcm"]
    for key, field in (("channels", "channel_count"), ("sample_rate", "sample_rate_hz")):
        require(active[key], summary[field], "measured " + key)
