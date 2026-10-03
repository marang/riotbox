"""Independent validation of the fixed silent-host observation transcript."""

import hashlib
import json

PROTOCOL_SHA256 = "9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea"
SAMPLE_SCHEMA = "riotbox.silent_host_sample.v1"


class EvidenceError(ValueError):
    pass


def load_protocol(path):
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != PROTOCOL_SHA256:
        raise EvidenceError("frozen V2 protocol bytes changed")
    return json.loads(data)


def integer(value, name):
    if type(value) is not int or value < 0:
        raise EvidenceError(f"invalid nonnegative integer: {name}")
    return value


def records_from_bytes(data, *, complete=False):
    if len(data) > 262144:
        raise EvidenceError("driver transcript exceeds bounded record budget")
    if complete and data and not data.endswith(b"\n"):
        raise EvidenceError("driver transcript ends with an incomplete record")
    lines = data.split(b"\n")[:-1]
    if len(lines) > 62:
        raise EvidenceError("driver emitted too many records")
    try:
        records = [json.loads(line) for line in lines]
    except (ValueError, UnicodeDecodeError) as error:
        raise EvidenceError("driver emitted malformed JSON") from error
    if any(not isinstance(record, dict) for record in records):
        raise EvidenceError("driver records must be objects")
    return records


def validate_records(records, pid, protocol, *, complete=False):
    """Validate a prefix; return whether all observation intervals have arrived.

    A complete successful transcript additionally requires the stopped record.
    Any explicit failure or malformed prefix aborts immediately.
    """
    count = protocol["run_seconds"] * 1000 // protocol["sample_interval_ms"]
    previous_count = None
    previous_elapsed = None
    geometry = None
    for index, record in enumerate(records):
        expected_event = "started" if index == 0 else "sample" if index <= count else "stopped"
        expected_index = min(index, count)
        if (index > count + 1 or record.get("schema") != SAMPLE_SCHEMA
                or integer(record.get("pid"), "pid") != pid
                or record.get("event") != expected_event
                or integer(record.get("sample_index"), "sample_index") != expected_index):
            raise EvidenceError("driver record identity/order mismatch")
        expected_result = "observing" if index == 0 else "ok"
        if record.get("result") != expected_result:
            raise EvidenceError(f"driver observation failed: {record.get('reason', '<no detail>')}")
        health = record.get("health")
        if not isinstance(health, dict):
            raise EvidenceError("driver health must be an object")
        lifecycle = "stopped" if expected_event == "stopped" else "running"
        if health.get("lifecycle") != lifecycle:
            raise EvidenceError("driver lifecycle does not match observation phase")
        for key in ("stream_error_count", "callback_scratch_overflow_count"):
            if integer(health.get(key), key) != 0:
                raise EvidenceError(f"driver observed {key}")
        if "last_stream_error" not in health or health["last_stream_error"] is not None:
            raise EvidenceError("driver retained stream error detail")
        current_geometry = tuple(health.get(key) for key in (
            "host_name", "device_name", "sample_format", "sample_rate", "channel_count"))
        if (not all(isinstance(value, str) and value for value in current_geometry[:3])
                or integer(current_geometry[3], "sample_rate") == 0
                or integer(current_geometry[4], "channel_count") != 2):
            raise EvidenceError("driver output geometry is absent or not stereo")
        if geometry is not None and current_geometry != geometry:
            raise EvidenceError("driver output geometry changed")
        geometry = current_geometry
        callbacks = integer(health.get("callback_count"), "callback_count")
        elapsed = integer(record.get("elapsed_ms"), "elapsed_ms")
        gap = health.get("max_callback_gap_micros")
        if gap is not None:
            integer(gap, "max_callback_gap_micros")
        if expected_event == "sample":
            if callbacks <= previous_count:
                raise EvidenceError("driver callback count did not advance")
            if elapsed - previous_elapsed < protocol["sample_interval_ms"]:
                raise EvidenceError("driver observation interval is too short")
        elif expected_event == "stopped":
            if callbacks < previous_count or elapsed < previous_elapsed:
                raise EvidenceError("driver terminal evidence went backwards")
            if elapsed - previous_elapsed >= protocol["teardown_deadline_seconds"] * 1000:
                raise EvidenceError("driver terminal teardown exceeded its budget")
        previous_count, previous_elapsed = callbacks, elapsed
    if complete and len(records) != count + 2:
        raise EvidenceError("driver exited without complete stopped evidence")
    return len(records) >= count + 1


def validate_preflight(data):
    fields = {}
    for line in data.decode("utf-8").splitlines():
        key, separator, value = line.partition(": ")
        if not separator or key in fields:
            raise EvidenceError("invalid or duplicate preflight diagnostic")
        fields[key] = value
    if fields.get("stream_result") != "Ok":
        raise EvidenceError("CPAL admission probe did not pass")
    try:
        callbacks = int(fields["callback_count"])
        errors = int(fields["stream_error_count"])
        overflow = int(fields["callback_scratch_overflow_count"])
    except (KeyError, ValueError) as error:
        raise EvidenceError("CPAL admission counters are missing") from error
    if callbacks <= 0 or errors != 0 or overflow != 0:
        raise EvidenceError("CPAL admission counters did not pass")
    return fields
