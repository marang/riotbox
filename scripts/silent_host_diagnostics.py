"""QA-only V4 ALSA measurement adapter; never grants host execution authority."""

import hashlib
import json
import os
from pathlib import Path
import platform
import re
import struct
import sys

from silent_host_evidence import EvidenceError, V3_PROTOCOL_SHA256, load_protocol_v3

V4_PROTOCOL_SHA256 = "36b23fc49def340150c0c63d54ab36b6fb0cc5272249523098d725fea4f7f740"
CAPACITY = 64
HEADER = struct.Struct("<8Q")
PCM = struct.Struct("<29Q")
SIZE = HEADER.size + CAPACITY * PCM.size
CALL_FIELDS = ("count", "negative_count", "first_negative", "epipe_count", "first_epipe_ns",
               "last_start_ns", "max_start_gap_ns", "max_call_duration_ns", "max_requested_frames")


def load_protocol_v4(root):
    path = root / "docs/benchmarks/silent_host_observation_v4.json"
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != V4_PROTOCOL_SHA256:
        raise EvidenceError("frozen V4 protocol bytes changed")
    protocol = json.loads(data)
    previous = load_protocol_v3(root / "docs/benchmarks/silent_host_observation_v3.json")
    changed = {"schema", "schema_version", "predecessor_protocol_sha256", "evidence_scope"}
    if (protocol["predecessor_protocol_sha256"] != V3_PROTOCOL_SHA256
            or any(protocol.get(key) != value for key, value in previous.items() if key not in changed)):
        raise EvidenceError("V4 changed predecessor safety/budgets")
    return protocol


def _signed(value):
    return value - (1 << 64) if value >= (1 << 63) else value


def decode_measurements(data, pid, *, complete):
    if len(data) != SIZE:
        raise EvidenceError("invalid diagnostic file size")
    magic, version, observed_pid, used, lost, ready, clock_failures, start = HEADER.unpack_from(data)
    if (magic != int.from_bytes(b"RBXALSA1", "little") or version != 1 or observed_pid != pid
            or ready != 1 or not 0 < used <= CAPACITY or lost or clock_failures or not start):
        raise EvidenceError("diagnostic identity/coverage/clock is incomplete")
    pcms = []
    for index in range(used):
        values = PCM.unpack_from(data, HEADER.size + index * PCM.size)
        handle, closed, params_calls, params_error, buffer, period, channels, rate = values[:8]
        if not handle or closed not in (0, 1):
            raise EvidenceError("invalid PCM lifetime identity")
        calls = {}
        for name, offset in (("avail", 8), ("write", 17)):
            measurement = dict(zip(CALL_FIELDS, values[offset:offset + 9]))
            measurement["first_negative"] = _signed(measurement["first_negative"])
            count, negatives, epipes = (measurement[key] for key in ("count", "negative_count", "epipe_count"))
            if (epipes > negatives or negatives > count
                    or bool(negatives) != (measurement["first_negative"] < 0)
                    or bool(epipes) != bool(measurement["first_epipe_ns"])
                    or bool(count) != bool(measurement["last_start_ns"])
                    or (epipes and measurement["first_epipe_ns"] < start)):
                raise EvidenceError("inconsistent diagnostic call counters")
            calls[name] = measurement
        pcms.append({"generation": index + 1, "closed": bool(closed), "params_calls": params_calls,
                     "params_error": _signed(params_error), "buffer_frames": buffer,
                     "period_frames": period, "channels": channels, "sample_rate": rate,
                     "calls": calls, "prepare_calls": values[26], "recover_calls": values[27],
                     "max_latest_avail_to_write_ns": values[28]})
    writing = [pcm for pcm in pcms if pcm["calls"]["write"]["count"]]
    if len(writing) != 1:
        raise EvidenceError("exactly one writing playback PCM must be measured")
    active = writing[0]
    if (not active["params_calls"] or active["params_error"] or active["channels"] != 2
            or not active["sample_rate"] or not 0 < active["period_frames"] <= active["buffer_frames"]
            or not active["calls"]["avail"]["count"] or (complete and not active["closed"])):
        raise EvidenceError("active PCM geometry/coverage/close is incomplete")
    if complete and any(call["negative_count"] for pcm in pcms for call in pcm["calls"].values()):
        raise EvidenceError("diagnostic ALSA error contradicts a successful observation")
    return {"schema": "riotbox.alsa_boundary_measurements.v1", "pid": pid,
            "start_monotonic_ns": start, "pcm_count": used, "active_pcm": active, "pcms": pcms,
            "complete_lifetime": active["closed"], "instrumented": True}


class AlsaDiagnostics:
    """One fixed measurement leaf attached to the existing supervised child."""

    def __init__(self, root, library, expected_hash):
        self.protocol = load_protocol_v4(root)
        self.library = Path(library)
        self.expected_hash = expected_hash
        if sys.platform != "linux" or platform.machine() != "x86_64" or sys.byteorder != "little":
            raise EvidenceError("unsupported diagnostic ABI")
        if (not self.library.is_absolute() or any(c in str(self.library) for c in " :\n\r\0")
                or len(expected_hash) != 64):
            raise EvidenceError("invalid diagnostic library binding")
        self.verify_library()

    def verify_library(self):
        if hashlib.sha256(self.library.read_bytes()).hexdigest() != self.expected_hash:
            raise EvidenceError("reviewed diagnostic library changed")

    def wrap(self, command, prefix, environment):
        self.verify_library()
        if any(key.startswith("LD_") or key == "RIOTBOX_ALSA_DIAGNOSTICS" for key in environment):
            raise EvidenceError("watchdog environment contains injected diagnostics")
        path = prefix.with_suffix(".alsa.bin")
        if not path.is_absolute() or any(c in str(path) for c in "\n\r\0"):
            raise EvidenceError("diagnostic owner path is invalid")
        fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        with os.fdopen(fd, "wb") as output:
            output.write(bytes(SIZE))
        # env exec preserves the timeout direct-child PID; timeout is NOT preloaded.
        return ["/usr/bin/env", f"LD_PRELOAD={self.library}",
                f"RIOTBOX_ALSA_DIAGNOSTICS={path}", *command]

    def finish(self, prefix, pid, *, complete):
        path = prefix.with_suffix(".alsa.bin")
        with path.open("rb") as source:
            data = source.read(SIZE + 1)
        report_path = prefix.with_suffix(".alsa.json")
        try:
            report = decode_measurements(data, pid, complete=complete)
        except EvidenceError as error:
            report = {"schema": "riotbox.alsa_boundary_measurements.v1", "pid": pid,
                      "instrumented": True, "validation_error": str(error)}
            self._publish(report_path, report, data)
            raise
        self._publish(report_path, report, data)
        return report

    @staticmethod
    def verify_geometry(report, result, *, preflight):
        if preflight:
            match = re.fullmatch(r"[^,]+, channels=([0-9]+), sample_rate=([0-9]+), buffer_size=.+",
                                 result.get("default_output_config", ""))
            if match is None:
                raise EvidenceError("preflight output geometry is not attributable")
            channels, rate = map(int, match.groups())
        else:
            channels = result["first"]["health"]["channel_count"]
            rate = result["first"]["health"]["sample_rate"]
        active = report["active_pcm"]
        if (channels, rate) != (active["channels"], active["sample_rate"]):
            raise EvidenceError("driver and measured PCM geometry disagree")

    @staticmethod
    def _publish(path, report, data):
        report["raw_sha256"] = hashlib.sha256(data).hexdigest()
        with path.open("x", encoding="utf-8") as output:
            json.dump(report, output, indent=2, allow_nan=False)
            output.write("\n")
