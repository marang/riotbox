"""Pulse module identity and cleanup for the source-free V2 operator seam.

Short text contains the module index that PulseAudio's module JSON omits.
Metadata queries and unload are separate commands, never an atomic compare-and-
unload transaction: observed replacements are rejected, not inter-command races.
"""

import re
import shlex
import time

from silent_host_evidence import EvidenceError
from silent_host_routes import RouteError, indexed, props


def _read_modules(host):
    modules = []
    seen = set()
    text = host.text(["pactl", "--format=text", "list", "short", "modules"])
    if not isinstance(text, str):
        raise EvidenceError("short module metadata must be text")
    for line in text.splitlines():
        fields = line.split("\t")
        if len(fields) == 4 and fields[-1] == "":
            fields.pop()
        if len(fields) != 3:
            raise EvidenceError("malformed short module columns")
        index, name, argument = fields
        if re.fullmatch(r"0|[1-9][0-9]{0,9}", index) is None:
            raise EvidenceError("module index is not canonical unsigned decimal")
        index = _module_index(int(index))
        if index in seen or not name:
            raise EvidenceError("duplicate module index or missing module name")
        seen.add(index)
        modules.append({"index": index, "name": name, "argument": argument})
    return modules


def _module_index(value):
    # Pulse indices are uint32; UINT32_MAX is PA_INVALID_INDEX, not an owner.
    if type(value) is not int or not 0 <= value < 0xFFFFFFFF:
        raise EvidenceError("invalid Pulse module index")
    return value


def _owned(modules, name):
    expected = {f"sink_name={name}", "channels=2", "channel_map=front-left,front-right"}
    matches = []
    for module in modules:
        try:
            arguments = shlex.split(module["argument"])
        except ValueError as error:
            raise EvidenceError("malformed short module arguments") from error
        sink_names = [argument.partition("=")[2] for argument in arguments
                      if argument.startswith("sink_name=")]
        if len(sink_names) > 1:
            raise EvidenceError("conflicting or duplicate sink_name arguments")
        if sink_names == [name]:
            if (module["name"] != "module-null-sink"
                    or len(arguments) != len(expected) or set(arguments) != expected):
                raise EvidenceError("owned module type or stereo arguments changed")
            matches.append(module)
    if len(matches) > 1:
        raise EvidenceError("owned null-sink module is ambiguous; not unloading")
    return matches


def owned_modules(host, name):
    """Return exact owned module records with their verified short-text index."""
    return _owned(_read_modules(host), name)


def _remaining(deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise EvidenceError("owned null-sink teardown deadline exceeded")
    return remaining


def _sink_present(snapshot, name):
    """Only well-formed Node metadata can establish named sink absence."""
    try:
        nodes = []
        for item in indexed(snapshot).values():
            interface = item.get("type")
            if (not isinstance(interface, str) or not interface.startswith("PipeWire:Interface:")
                    or not interface.removeprefix("PipeWire:Interface:")):
                raise RouteError("sink cleanup observation has no valid object type")
            if interface == "PipeWire:Interface:Node":
                node_name = props(item).get("node.name")
                if not isinstance(node_name, str) or not node_name:
                    raise RouteError("sink cleanup Node has no valid name")
                nodes.append(node_name)
        return name in nodes
    except RouteError as error:
        raise EvidenceError(f"invalid sink cleanup metadata: {error}") from error


def remove_owned_sink(host, name, module_id, protocol):
    """Unload at most once, then require both module and named Node absence.

    The total budget includes pre-unload queries. Host methods retain their own
    command bounds; an over-budget response cannot authorize unload or success.
    """
    if module_id is not None:
        _module_index(module_id)
    deadline = time.monotonic() + protocol["teardown_deadline_seconds"]
    unloaded = False
    while True:
        _remaining(deadline)
        modules = _read_modules(host)
        _remaining(deadline)
        matches = _owned(modules, name)
        if module_id is not None:
            bound = next((module for module in modules if module["index"] == module_id), None)
            if bound is not None and bound not in matches:
                raise EvidenceError("owned module ID now identifies a replacement; not unloading")
            if matches and matches[0]["index"] != module_id:
                raise EvidenceError("owned null-sink module ID changed; not unloading")
        _remaining(deadline)
        if matches and not unloaded:
            actual_id = matches[0]["index"]
            module_id = actual_id
            host.text(["pactl", "unload-module", str(actual_id)])
            _remaining(deadline)
            unloaded = True
            continue
        sink_present = _sink_present(host.snapshot(), name)
        _remaining(deadline)
        if not matches and not sink_present:
            return
        time.sleep(min(protocol["route_poll_interval_ms"] / 1000, _remaining(deadline)))
