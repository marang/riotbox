"""Closed loaded-recorder supervision; this module grants no execution authority."""

from dataclasses import asdict
import json
from pathlib import Path
import time

from loaded_recorder_evidence import LoadedObservationVersion
from silent_host_evidence import EvidenceError
from silent_host_process import ManagedProcess
from silent_host_routes import indexed, kind, node_present, number, observe_route, props

NANOSECONDS = 1_000_000_000
MAX_STDOUT_BYTES = 262144


def _monotonic_ns():
    return time.clock_gettime_ns(time.CLOCK_MONOTONIC)


def _snapshot(host, snapshots, pid, phase):
    start = _monotonic_ns()
    try:
        objects = host.snapshot()
    except BaseException as error:
        snapshots.write(json.dumps({
            "phase": phase, "pid": pid, "clock_domain": "linux_clock_monotonic",
            "query_start_monotonic_ns": start, "query_end_monotonic_ns": _monotonic_ns(),
            "error": f"{type(error).__name__}: {error}",
        }) + "\n")
        snapshots.flush()
        raise
    end = _monotonic_ns()
    if end < start:
        raise EvidenceError("route query monotonic clock moved backwards")
    snapshots.write(json.dumps({
        "phase": phase, "pid": pid, "clock_domain": "linux_clock_monotonic",
        "query_start_monotonic_ns": start, "query_end_monotonic_ns": end,
        "objects": objects,
    }) + "\n")
    snapshots.flush()
    return objects, start, end


def _check_absence(absence, marker):
    if absence is None or marker is None:
        return
    start, end = absence
    complete = marker["window_complete_monotonic_ns"]
    if end < complete:
        raise EvidenceError("admitted route disappeared before window completion")
    if start < complete:
        raise EvidenceError("first absent route query overlaps window completion; ambiguous")


def _check_admitted_node(objects, identity, version):
    if node_present(objects, identity):
        return
    occupant = indexed(objects).get(identity.node_id)
    if occupant is None:
        return
    if version is LoadedObservationVersion.V2 and kind(occupant, "Client"):
        # Only this closed non-audio interface is qualified by V2. It remains
        # absence, never completion or permission to change a process route.
        serial = number(props(occupant).get("object.serial"))
        if 0 < serial < 2**64 and serial != identity.serial:
            return
    raise EvidenceError("admitted loaded node identity changed")


def run_recording(binary, case, fixture, prefix, host, sink, protocol, environment, *,
                  diagnostics, observation_version=LoadedObservationVersion.V1):
    """Run exactly the existing recorder CLI through established containment.

    Callers must already admit the frozen fixture and own the declared case paths.
    There is no command/profile seam, executable CLI or host authorization here.
    """
    if type(observation_version) is not LoadedObservationVersion:
        raise EvidenceError("unsupported loaded observation version")
    failure = None
    result = None
    try:
        fixture.verify_unchanged()
        if diagnostics is None:
            raise EvidenceError("loaded recording requires the bound ALSA diagnostics")
        if type(case.bars) is not int or case.bars not in (2, 8, 16):
            raise EvidenceError("unsupported loaded recording duration")
        result = _run_recording(binary, case, fixture, Path(prefix), host, sink,
                                protocol, environment, diagnostics, observation_version)
    except BaseException as error:
        failure = error
    # Always perform the immutable-input check, including interrupted/failed
    # supervision. Preserve typed containment failures rather than replacing them.
    try:
        fixture.verify_unchanged()
    except BaseException as error:
        if failure is None:
            failure = error
        else:
            failure.add_note(f"fixture preservation failed: {type(error).__name__}: {error}")
    if failure is not None:
        raise failure
    return result


def _run_recording(binary, case, fixture, prefix, host, sink, protocol, environment, diagnostics,
                   observation_version):
    # Local imports keep the closed workload adapter independent of the operator's
    # orchestration import direction. All lifecycle/cleanup owners remain shared.
    from loaded_recorder_evidence import (
        load_json_bytes, read_observer, validate_final, verify_loaded_geometry,
    )
    from run_silent_host_probe import (
        UnverifiedCleanup, UnverifiedGroupCleanup, UnverifiedStreamCleanup,
        digest, require_removed, route_snapshots, write_json,
    )

    command = [str(binary), "--live-master-recording-execute",
               "--session", str(case.session_path),
               "--live-recording-destination", str(case.destination_path),
               "--live-recording-bars", str(case.bars),
               "--observer", str(case.observer_path)]
    stdout = prefix.with_suffix(".stdout.log")
    stderr = prefix.with_suffix(".stderr.log")
    routes = prefix.with_suffix(".routes.ndjson.gz")
    baseline = host.default_state()
    start_ns = _monotonic_ns()
    startup_deadline = start_ns + protocol["startup_route_deadline_seconds"] * NANOSECONDS
    watchdog_deadline = start_ns + protocol["process_timeout_seconds"] * NANOSECONDS
    teardown_ns = protocol["teardown_deadline_seconds"] * NANOSECONDS
    pid = None
    observed = None
    marker = None
    absence = None
    terminal_deadline = None
    route_count = 0
    failure = None
    summary = None
    artifact_evidence = None
    measurement = None
    command = diagnostics.wrap(command, prefix, environment)
    managed = ManagedProcess(command, stdout, stderr, environment,
                             protocol["process_timeout_seconds"], protocol["kill_grace_seconds"])

    def unchanged_default():
        if host.default_state() != baseline:
            raise EvidenceError("default output/mute/volume changed during loaded recording")

    def refresh_marker(*, complete=False):
        nonlocal marker, terminal_deadline
        current = read_observer(case.observer_path, pid, case, fixture, complete=complete)
        if current is None:
            if marker is not None:
                raise EvidenceError("observed stopped-window milestone disappeared")
            if complete:
                raise EvidenceError("process exited without a stopped-window milestone")
            return
        if marker is not None and marker != current:
            raise EvidenceError("observed stopped-window milestone changed")
        if (current["window_complete_monotonic_ns"] < start_ns
                or current["runtime_stopped_monotonic_ns"] > _monotonic_ns()):
            raise EvidenceError("stopped-window milestone has stale or future timestamps")
        marker = current
        stopped_deadline = current["runtime_stopped_monotonic_ns"] + teardown_ns
        terminal_deadline = (stopped_deadline if terminal_deadline is None
                             else min(terminal_deadline, stopped_deadline))
        _check_absence(absence, marker)

    def check_deadlines():
        now = _monotonic_ns()
        if terminal_deadline is not None and now >= terminal_deadline:
            raise EvidenceError("loaded recording terminal teardown deadline exceeded")
        if now >= watchdog_deadline:
            raise EvidenceError("loaded recording independent watchdog budget exceeded")

    with route_snapshots(routes) as snapshots:
        try:
            with managed as process:
                while True:
                    check_deadlines()
                    unchanged_default()
                    if pid is None:
                        pid = process.child_pid
                        if pid is None:
                            if process.exited():
                                raise EvidenceError("process exited before child identity was bound")
                            if _monotonic_ns() >= startup_deadline:
                                raise EvidenceError("loaded recorder child identity missed startup deadline")
                            time.sleep(0.005)
                            continue
                    refresh_marker()
                    check_deadlines()
                    objects, query_start, query_end = _snapshot(host, snapshots, pid, "observe")
                    # Unsafe route/identity errors are never a pending state.
                    route = observe_route(objects, sink, pid)
                    if observed is not None:
                        _check_admitted_node(objects, observed.node, observation_version)
                    unchanged_default()
                    refresh_marker()
                    check_deadlines()
                    if observed is None and _monotonic_ns() >= startup_deadline:
                        raise EvidenceError("first loaded route arrived after the startup deadline")
                    if route is not None:
                        if absence is not None:
                            raise EvidenceError("admitted loaded route reappeared after terminal absence")
                        if observed is not None and observed != route:
                            raise EvidenceError("admitted process route changed")
                        observed = route
                        route_count += 1
                    elif observed is not None:
                        if absence is None:
                            absence = (query_start, query_end)
                            absent_deadline = query_start + teardown_ns
                            terminal_deadline = (absent_deadline if terminal_deadline is None
                                                 else min(terminal_deadline, absent_deadline))
                        _check_absence(absence, marker)
                        check_deadlines()
                    if process.exited():
                        status = process.finish()
                        if status != 0:
                            raise EvidenceError(f"observed process exited with status {status}")
                        refresh_marker(complete=True)
                        check_deadlines()
                        if observed is None:
                            raise EvidenceError("process exited without observed loaded route identity")
                        with stdout.open("rb") as data:
                            payload = data.read(MAX_STDOUT_BYTES + 1)
                        if len(payload) > MAX_STDOUT_BYTES:
                            raise EvidenceError("loaded recorder stdout exceeds 262144 bytes")
                        summary = load_json_bytes(payload)
                        check_deadlines()
                        break
                    if observed is None and _monotonic_ns() >= startup_deadline:
                        raise EvidenceError("exact loaded route was not admitted within startup budget")
                    time.sleep(protocol["route_poll_interval_ms"] / 1000)
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
                    class CleanupHost:
                        def snapshot(self):
                            unchanged_default()
                            objects, _, _ = _snapshot(host, snapshots, pid, "teardown_query")
                            unchanged_default()
                            return objects

                    # Both clocks are Linux CLOCK_MONOTONIC. Keep the actual
                    # stopped/first-absence anchor; cleanup cannot reset its budget.
                    cleanup_deadline = (terminal_deadline if terminal_deadline is not None
                                        else _monotonic_ns() + teardown_ns)
                    require_removed(CleanupHost(), sink, pid, protocol, snapshots,
                                    deadline=cleanup_deadline / NANOSECONDS,
                                    node=observed.node if observed else None)
                except BaseException as error:
                    if not isinstance(failure, UnverifiedCleanup):
                        original = failure
                        failure = UnverifiedStreamCleanup(
                            f"stream removal unverified: pid {pid}, node "
                            f"{observed.node.node_id if observed else 'unadmitted'}")
                        if original is not None:
                            failure.add_note(f"original failure: {type(original).__name__}: {original}")
                            for note in getattr(original, "__notes__", []):
                                failure.add_note(note)
                    failure.add_note(f"stream cleanup not verified: {error}")
        # The fixed terminal budget bounds process/stream removal, not an
        # independent Python sample-analysis pass after all audio has stopped.
        # Observe cleanup first; a surviving writer cannot supply stable files.
        if failure is None:
            try:
                artifact_evidence = validate_final(summary, marker, case, fixture)
                write_json(prefix.with_suffix(".artifacts.json"), {
                    "result": "pass", "evidence": artifact_evidence,
                })
            except EvidenceError as error:
                if hasattr(error, "artifact_evidence"):
                    try:
                        write_json(prefix.with_suffix(".artifacts.json"), {
                            "result": "failed", "error": str(error),
                            "evidence": error.artifact_evidence,
                        })
                    except BaseException as report_error:
                        error.add_note(f"artifact evidence publication failed: {report_error}")
                failure = error
            except BaseException as error:
                failure = error
        if managed.cleanup_verified and pid is not None:
            try:
                measurement = diagnostics.finish(prefix, pid, complete=failure is None)
                if failure is None:
                    verify_loaded_geometry(diagnostics, measurement, summary)
            except BaseException as error:
                if failure is None:
                    failure = error
                else:
                    failure.add_note(f"diagnostic validation failed: {type(error).__name__}: {error}")
        if failure is not None:
            raise failure
    return {"pid": pid, "route": asdict(observed), "route_observations": route_count,
            "elapsed_seconds": (_monotonic_ns() - start_ns) / NANOSECONDS,
            "result": summary, "marker": marker, "artifact_evidence": artifact_evidence,
            "stdout_sha256": digest(stdout), "observer_sha256": digest(case.observer_path),
            "routes_sha256": digest(routes), "alsa_measurements": measurement}
