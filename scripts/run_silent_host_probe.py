#!/usr/bin/env python3
"""Source-free V2 lifecycle checks; host execution has no active CLI entry point.

V1 is consumed. A future host attempt requires its own prospective phase/owner.
"""

import argparse
from contextlib import contextmanager
from dataclasses import asdict
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid

from silent_host_evidence import (
    EvidenceError, PROTOCOL_SHA256, load_protocol, records_from_bytes,
    validate_preflight, validate_records,
)
from silent_host_environment import local_host_environment, prepare_child_environment
from silent_host_modules import owned_modules, remove_owned_sink
from silent_host_process import ManagedProcess
from silent_host_routes import NodeIdentity, attached_streams, find_sink, node_present, observe_route

ROOT = Path(__file__).resolve().parents[1]


class UnverifiedCleanup(EvidenceError):
    """Keep the containment sink while owned process/stream removal is uncertain."""


class UnverifiedGroupCleanup(UnverifiedCleanup):
    pass


class UnverifiedStreamCleanup(UnverifiedCleanup):
    pass


def write_json(path, value):
    with path.open("x", encoding="utf-8") as output:
        json.dump(value, output, indent=2, allow_nan=False)
        output.write("\n")


def write_result(path, attempt, interrupts):
    """Publish this owner's result, preserving a signal received during writing.

    A just-created, identity-checked file may correct its provisional contents;
    an earlier attempt/result is never opened for replacement. This is operational
    evidence, not a crash-durable transaction or a new Session persistence path.
    """
    with path.open("x", encoding="utf-8") as output:
        identity = os.fstat(output.fileno())
        json.dump(attempt, output, indent=2, allow_nan=False)
        output.write("\n")
        output.flush()
    # Include flush AND close in publication. A latched signal can arrive in
    # either; completion is considered only after the closed file is checked.
    if interrupts.received is not None and "interruption" not in attempt:
        attempt["result"] = "failed"
        attempt["interruption"] = f"operator interrupted by signal {interrupts.received} during publication"
        with path.open("r+", encoding="utf-8") as output:
            current = os.fstat(output.fileno())
            if (identity.st_dev, identity.st_ino) != (current.st_dev, current.st_ino):
                raise EvidenceError("result identity changed before interruption correction")
            json.dump(attempt, output, indent=2, allow_nan=False)
            output.write("\n")
            output.truncate()


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


class HostCommands:
    """The external-command seam: all metadata operations are time-bounded."""

    def __init__(self, timeout, environment=None):
        self.timeout = timeout
        self.environment = environment

    def text(self, command):
        result = subprocess.run(command, cwd=ROOT, capture_output=True,
                                text=True, timeout=self.timeout, check=True,
                                env=self.environment)
        if len(result.stdout) > 16 * 1024 * 1024:
            raise EvidenceError("metadata response is too large")
        # Short Pulse module output includes meaningful empty tab columns.
        return result.stdout.rstrip("\r\n")

    def json(self, command):
        return json.loads(self.text(command))

    def snapshot(self):
        return self.json(["pw-dump"])

    def default_state(self):
        info = self.json(["pactl", "--format=json", "info"])
        sinks = self.json(["pactl", "--format=json", "list", "sinks"])
        matches = [sink for sink in sinks if sink["name"] == info["default_sink_name"]]
        if len(matches) != 1:
            raise EvidenceError("current default sink is missing or ambiguous")
        sink = matches[0]
        return {"default_sink_name": sink["name"], "mute": sink["mute"],
                "volume": sink["volume"]}


def read_transcript(path, *, complete=False):
    with path.open("rb") as data:
        return records_from_bytes(data.read(262145), complete=complete)


@contextmanager
def route_snapshots(path):
    """Close the owned log without replacing a primary containment failure."""
    snapshots = gzip.open(path, "xt", encoding="utf-8")
    try:
        yield snapshots
    except BaseException as primary:
        try:
            snapshots.close()
        except BaseException as close_error:
            primary.add_note(f"route log close failed: {type(close_error).__name__}: {close_error}")
        raise
    else:
        snapshots.close()


def stream_present(snapshot, pid):
    clients = {item["id"] for item in snapshot
               if item.get("type") == "PipeWire:Interface:Client"
               and str(item.get("info", {}).get("props", {}).get("application.process.id"))
               == str(pid)}
    return any(item.get("type") == "PipeWire:Interface:Node"
               and str(item.get("info", {}).get("props", {}).get("client.id"))
               in {str(value) for value in clients} for item in snapshot)


def require_removed(host, sink, pid, protocol, snapshots, *, deadline=None,
                    node: NodeIdentity | None = None):
    if deadline is None:
        deadline = time.monotonic() + protocol["teardown_deadline_seconds"]
    while True:
        snapshot = host.snapshot()
        snapshots.write(json.dumps({"phase": "teardown", "monotonic": time.monotonic(),
                                    "objects": snapshot}) + "\n")
        # Identity and no outgoing sink/foreign links remain mandatory at exit.
        observe_route(snapshot, sink, pid)
        if time.monotonic() >= deadline:
            raise EvidenceError(f"terminal teardown deadline exceeded: pid {pid}")
        node_remains = node is not None and node_present(snapshot, node)
        if not node_remains and not attached_streams(snapshot, sink) and not stream_present(snapshot, pid):
            return
        time.sleep(protocol["route_poll_interval_ms"] / 1000)


def run_program(command, prefix, host, sink, protocol, environment, *, preflight=False):
    """Own one process and continuously observe the exact external route."""
    stdout = prefix.with_suffix(".stdout.log" if preflight else ".stdout.ndjson")
    stderr = prefix.with_suffix(".stderr.log")
    watchdog = (protocol["preflight_process_timeout_seconds"] if preflight
                else protocol["process_timeout_seconds"])
    pid = None
    observed = None
    route_count = 0
    start = time.monotonic()
    failure = None
    result = None
    terminal_deadline = None
    interval_count = protocol["run_seconds"] * 1000 // protocol["sample_interval_ms"]
    managed = ManagedProcess(command, stdout, stderr, environment, watchdog,
                             protocol["kill_grace_seconds"])
    with route_snapshots(prefix.with_suffix(".routes.ndjson.gz")) as snapshots:
        try:
            with managed as process:
                while True:
                    if pid is None:
                        pid = process.child_pid
                        if pid is None:
                            if process.exited():
                                raise EvidenceError("process exited before child identity was bound")
                            if time.monotonic() - start >= protocol["startup_route_deadline_seconds"]:
                                raise EvidenceError("process child identity was not established")
                            time.sleep(0.005)
                            continue
                    complete_intervals = False
                    if pid is not None and not preflight:
                        records = read_transcript(stdout)
                        complete_intervals = validate_records(records, pid, protocol)
                    if complete_intervals and terminal_deadline is None:
                        # The driver timer starts after this operator's start;
                        # this conservative anchor cannot grant extra time when
                        # metadata delays detection of the final sample.
                        terminal_deadline = (start + records[interval_count]["elapsed_ms"] / 1000
                                             + protocol["teardown_deadline_seconds"])
                    if terminal_deadline is not None and time.monotonic() >= terminal_deadline:
                        raise EvidenceError("terminal teardown deadline exceeded before process exit")
                    snapshot = host.snapshot()
                    snapshots.write(json.dumps({"phase": "observe", "monotonic": time.monotonic(),
                                                "pid": pid, "objects": snapshot}) + "\n")
                    snapshots.flush()
                    if observed is None and time.monotonic() - start >= protocol["startup_route_deadline_seconds"]:
                        raise EvidenceError("first route arrived after the startup deadline")
                    # A metadata command can overlap the final driver record;
                    # admit natural drain only against the refreshed transcript.
                    if not preflight:
                        records = read_transcript(stdout)
                        complete_intervals = validate_records(records, pid, protocol)
                        if complete_intervals and terminal_deadline is None:
                            terminal_deadline = (start + records[interval_count]["elapsed_ms"] / 1000
                                                 + protocol["teardown_deadline_seconds"])
                    if terminal_deadline is not None and time.monotonic() >= terminal_deadline:
                        raise EvidenceError("terminal teardown deadline exceeded during route observation")
                    route = observe_route(snapshot, sink, pid if pid is not None else -1)
                    if route is not None:
                        if observed is not None and observed != route:
                            raise EvidenceError("admitted process route changed")
                        observed = route
                        route_count += 1
                    elif observed is not None:
                        # Stream removal can race with final stdout/process exit.
                        # Accept it only after all intervals, or for the short
                        # preflight at natural process exit; final proof still gates.
                        if not complete_intervals and not (preflight and process.exited()):
                            raise EvidenceError("admitted route disappeared before completion")
                    if process.exited():
                        if terminal_deadline is None:
                            terminal_deadline = time.monotonic() + protocol["teardown_deadline_seconds"]
                        status = process.finish()
                        if time.monotonic() >= terminal_deadline:
                            raise EvidenceError("terminal teardown deadline exceeded during group cleanup")
                        if status != 0:
                            raise EvidenceError(f"observed process exited with status {status}")
                        break
                    if observed is None and time.monotonic() - start >= protocol["startup_route_deadline_seconds"]:
                        raise EvidenceError("exact process route was not admitted within startup budget")
                    if time.monotonic() - start > watchdog + protocol["kill_grace_seconds"] + 1:
                        raise EvidenceError("process supervisor exceeded its independent watchdog budget")
                    time.sleep(protocol["route_poll_interval_ms"] / 1000)
                if pid is None or observed is None:
                    raise EvidenceError("process exited without observed route identity")
                if preflight:
                    with stdout.open("rb") as data:
                        result = validate_preflight(data.read(262145))
                else:
                    records = read_transcript(stdout, complete=True)
                    validate_records(records, pid, protocol, complete=True)
                    if time.monotonic() - start < protocol["run_seconds"]:
                        raise EvidenceError("actual process lifetime is shorter than the frozen observation")
                    result = {"first": records[0], "last": records[-1],
                              "sample_count": len(records) - 2}
        except BaseException as error:
            failure = error
        finally:
            if not managed.cleanup_verified:
                original = failure
                failure = UnverifiedGroupCleanup("owned process group cleanup was not verified")
                if original is not None:
                    failure.add_note(f"original failure: {type(original).__name__}: {original}")
                    for note in getattr(original, "__notes__", []):
                        failure.add_note(note)
            if pid is not None:
                try:
                    require_removed(host, sink, pid, protocol, snapshots,
                                    deadline=terminal_deadline,
                                    node=observed.node if observed else None)
                except BaseException as cleanup_error:
                    if not isinstance(failure, UnverifiedCleanup):
                        original = failure
                        failure = UnverifiedStreamCleanup(
                            f"stream removal unverified: pid {pid}, node "
                            f"{observed.node.node_id if observed else 'unadmitted'}")
                        if original is not None:
                            failure.add_note(f"original failure: {type(original).__name__}: {original}")
                            for note in getattr(original, "__notes__", []):
                                failure.add_note(note)
                    failure.add_note(f"stream cleanup not verified: {cleanup_error}")
        if failure is not None:
            raise failure
    return {"pid": pid, "route": asdict(observed), "route_observations": route_count,
            "elapsed_seconds": time.monotonic() - start, "result": result,
            "stdout_sha256": digest(stdout),
            "routes_sha256": digest(prefix.with_suffix(".routes.ndjson.gz"))}


def execute_attempt(interrupts, *, owner):
    """Reserved orchestration seam; currently called only with generated adapters.

    There is deliberately no default owner and no CLI path into this function.
    A real-host caller requires a separately authorized prospective phase.
    """
    protocol = load_protocol(ROOT / "docs/benchmarks/silent_host_observation_v2.json")
    if sys.platform != "linux" or os.getuid() == 0:
        raise EvidenceError("requires a non-root real Linux user session")
    host_environment = local_host_environment(os.environ)
    host = HostCommands(protocol["metadata_timeout_seconds"], host_environment)
    for tool in ("pactl", "pw-dump", "timeout", "loginctl", "systemd-detect-virt", "git"):
        if shutil.which(tool) is None:
            raise EvidenceError(f"required host tool unavailable: {tool}")
    if host.text(["git", "status", "--porcelain"]):
        raise EvidenceError("host attempt requires a clean reviewed revision")
    binaries = {name: ROOT / "target/debug" / name for name in ("cpal_spike", "silent_host_probe")}
    hashes = {name: digest(path) for name, path in binaries.items()}
    owner.mkdir(parents=True, exist_ok=False)
    name = "riotbox_silent_host_v2_" + uuid.uuid4().hex
    module_id = None
    baseline = None
    sink = None
    last_pid = None
    creation_attempted = False
    attempt = {"schema": "riotbox.silent_host_attempt.v2", "result": "failed",
               "protocol_sha256": PROTOCOL_SHA256, "binary_sha256": hashes,
               "git_revision": host.text(["git", "rev-parse", "HEAD"]),
               "sink_name": name, "runs": [], "cleanup_verified": False}
    write_json(owner / "admission.json", attempt)
    failure = None
    try:
        # A container's visible host sockets alone do not establish real-session context.
        container = subprocess.run(["systemd-detect-virt", "--container"],
                                   capture_output=True, text=True,
                                   timeout=protocol["metadata_timeout_seconds"])
        session_id = os.environ.get("XDG_SESSION_ID")
        if container.returncode != 1 or container.stdout.strip() != "none" or not session_id:
            raise EvidenceError("non-container real-session context not established")
        session = host.text(["loginctl", "show-session", session_id, "-p", "Active", "-p", "Remote", "-p", "User"])
        fields = dict(line.split("=", 1) for line in session.splitlines())
        if fields != {"Active": "yes", "Remote": "no", "User": str(os.getuid())}:
            raise EvidenceError("active local user session not established")
        info = host.json(["pactl", "--format=json", "info"])
        if info.get("is_local") != "yes" or "PipeWire" not in info.get("server_name", ""):
            raise EvidenceError("local PipeWire endpoint not established")
        attempt["context"] = {"session": fields, "server": info["server_name"],
                              "host": info["host_name"], "uid": os.getuid()}
        baseline = host.default_state()
        attempt["baseline"] = baseline
        if owned_modules(host, name):
            raise EvidenceError("proposed owned sink already exists")
        creation_attempted = True
        module_id = int(host.text(["pactl", "load-module", "module-null-sink",
                                   f"sink_name={name}", "channels=2",
                                   "channel_map=front-left,front-right"]))
        attempt["module_id"] = module_id
        sink = find_sink(host.snapshot(), name)
        attempt["sink_node_id"], attempt["sink_serial"] = sink.node_id, sink.serial
        if host.default_state() != baseline:
            raise EvidenceError("default output changed during null-sink admission")
        environment = prepare_child_environment(owner, sink, host_environment)
        attempt["alsa_config_sha256"] = digest(Path(environment["ALSA_CONFIG_PATH"]))
        attempt["pipewire_remote"] = environment["PIPEWIRE_REMOTE"]
        attempt["preflight"] = run_program([str(binaries["cpal_spike"])], owner / "preflight",
                                           host, sink, protocol, environment, preflight=True)
        last_pid = attempt["preflight"]["pid"]
        for index in range(1, protocol["run_count"] + 1):
            if host.default_state() != baseline:
                raise EvidenceError("default output changed before the next run")
            result = run_program([str(binaries["silent_host_probe"]), "--isolated-silent-host"],
                                 owner / f"run-{index}", host, sink, protocol, environment)
            last_pid = result["pid"]
            attempt["runs"].append(result)
        if host.default_state() != baseline:
            raise EvidenceError("default output changed during the host attempt")
        attempt["result"] = "pass"
    except BaseException as error:
        failure = error
        attempt["error"] = f"{type(error).__name__}: {error}"
        attempt["error_notes"] = getattr(error, "__notes__", [])
    finally:
        with interrupts.cleanup():
            try:
                # run_program terminates/reaps its group before returning/raising.
                # Refuse sink removal if any stream remains attached to the owner.
                if isinstance(failure, UnverifiedCleanup):
                    raise EvidenceError("containment sink retained: process/stream cleanup is unverified")
                if sink is not None:
                    snapshot = host.snapshot()
                    route = observe_route(snapshot, sink, last_pid if last_pid is not None else -1)
                    if route is not None or attached_streams(snapshot, sink):
                        raise EvidenceError("owned sink still has a runtime stream")
                if creation_attempted:
                    remove_owned_sink(host, name, module_id, protocol)
                if baseline is not None and host.default_state() != baseline:
                    raise EvidenceError("original default/mute/volume state no longer matches")
                attempt["cleanup_verified"] = True
            except BaseException as cleanup_error:
                attempt["result"] = "failed"
                attempt["cleanup_error"] = f"{type(cleanup_error).__name__}: {cleanup_error}"
                if failure is None:
                    failure = cleanup_error
            try:
                interrupts.check()
            except KeyboardInterrupt as error:
                attempt["result"] = "failed"
                attempt["interruption"] = str(error)
                if failure is None:
                    failure = error
            write_result(owner / "result.json", attempt, interrupts)
    interrupts.check()
    if failure is not None:
        raise failure
    print(json.dumps({"owner": str(owner), "result": attempt["result"],
                      "runs": len(attempt["runs"]), "cleanup_verified": True}))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute-reviewed-attempt", action="store_true", required=True)
    parser.parse_args(argv)
    print("silent host execution blocked: V1 is consumed; V2 is source-free only. "
          "A new host attempt requires a separate prospective phase and owner.", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
