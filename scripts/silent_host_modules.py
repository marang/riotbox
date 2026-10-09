"""Pulse module identity and cleanup for the source-free V3 operator seam.

JSON payloads frame the short text containing genuine Pulse module indices.
Metadata queries and unload are separate commands, never an atomic compare-and-
unload transaction: observed replacements are rejected, not inter-command races.
"""

import io
import re
import shlex
import time
import unicodedata

from silent_host_evidence import EvidenceError
from silent_host_routes import RouteError, indexed, props


def _check_deadline(deadline):
    if deadline is not None:
        _remaining(deadline)


class _DeadlineArgumentStream(io.StringIO):
    """Check during one large shlex token, not only when tokens are yielded."""

    def __init__(self, argument, deadline):
        super().__init__(argument)
        self.deadline = deadline
        self.read_count = 0

    def read(self, size=-1):
        self.read_count += 1
        # shlex reads one character at a time. This cooperative batch is not
        # an admission limit; the existing absolute deadline remains authority.
        if self.read_count % 1024 == 0:
            _check_deadline(self.deadline)
        return super().read(size)


def _valid_name(name, deadline):
    if not isinstance(name, str) or not name:
        raise EvidenceError("missing or invalid module name")
    for offset in range(0, len(name), 1024):
        _check_deadline(deadline)
        if any(unicodedata.category(char) == "Cc" for char in name[offset:offset + 1024]):
            raise EvidenceError("module name contains control characters")


def _read_modules(host, *, deadline=None):
    """Frame one paired observation without ordering assumptions or recovery.

    JSON retains original argument strings; text-mode newline projection and
    HostCommands' terminal normalization are forward framing operations only.
    Optional deadlines exist for generated tests; production callers supply one.
    """
    _check_deadline(deadline)
    payloads = host.json(["pactl", "--format=json", "list", "short", "modules"])
    _check_deadline(deadline)
    text = host.text(["pactl", "--format=text", "list", "short", "modules"])
    _check_deadline(deadline)
    if not isinstance(payloads, list):
        raise EvidenceError("short module JSON must be an array")
    if not isinstance(text, str):
        raise EvidenceError("short module metadata must be text")
    candidates = {}
    projections = {}
    for payload in payloads:
        _check_deadline(deadline)
        if not isinstance(payload, dict) or not isinstance(payload.get("argument"), str):
            raise EvidenceError("invalid short module JSON payload")
        name, argument = payload.get("name"), payload["argument"]
        _valid_name(name, deadline)
        projection = argument.replace("\r\n", "\n").replace("\r", "\n")
        _check_deadline(deadline)
        projected_key = (name, projection)
        if projected_key in projections and projections[projected_key] != argument:
            raise EvidenceError("ambiguous module argument newline projection")
        projections[projected_key] = argument
        by_argument = candidates.setdefault(name, {})
        if argument not in by_argument:
            raw_frames = {projection + "\t\n", projection + "\n"}
            final_frames = {frame.rstrip("\r\n") for frame in raw_frames}
            by_argument[argument] = [0, raw_frames, final_frames]
        by_argument[argument][0] += 1
    modules = []
    seen = set()
    offset = 0
    while offset < len(text):
        _check_deadline(deadline)
        index_end = text.find("\t", offset, offset + 11)
        if index_end < 0:
            raise EvidenceError("malformed short module columns")
        index = text[offset:index_end]
        if re.fullmatch(r"0|[1-9][0-9]{0,9}", index) is None:
            raise EvidenceError("module index is not canonical unsigned decimal")
        index = _module_index(int(index))
        if index in seen:
            raise EvidenceError("duplicate module index")
        name_end = text.find("\t", index_end + 1)
        if name_end < 0:
            raise EvidenceError("malformed short module columns")
        name = text[index_end + 1:name_end]
        _valid_name(name, deadline)
        argument_start = name_end + 1
        matches = set()
        for argument, (count, raw_frames, final_frames) in candidates.get(name, {}).items():
            _check_deadline(deadline)
            if count == 0:
                continue
            for frame in raw_frames:
                _check_deadline(deadline)
                if text.startswith(frame, argument_start):
                    matches.add((argument, argument_start + len(frame)))
                    if len(matches) > 1:
                        raise EvidenceError("ambiguous short module payload framing")
            for frame in final_frames:
                _check_deadline(deadline)
                if argument_start + len(frame) == len(text) and text.startswith(frame, argument_start):
                    matches.add((argument, len(text)))
                    if len(matches) > 1:
                        raise EvidenceError("ambiguous short module payload framing")
        if not matches:
            raise EvidenceError("short module JSON/text payload mismatch")
        argument, offset = matches.pop()
        candidates[name][argument][0] -= 1
        seen.add(index)
        modules.append({"index": index, "name": name, "argument": argument})
    for by_argument in candidates.values():
        for count, _raw, _final in by_argument.values():
            _check_deadline(deadline)
            if count:
                raise EvidenceError("short module JSON/text multiset mismatch")
    _check_deadline(deadline)
    return modules


def _module_index(value):
    # Pulse indices are uint32; UINT32_MAX is PA_INVALID_INDEX, not an owner.
    if type(value) is not int or not 0 <= value < 0xFFFFFFFF:
        raise EvidenceError("invalid Pulse module index")
    return value


def _owned(modules, name, *, deadline=None):
    expected = {f"sink_name={name}", "channels=2", "channel_map=front-left,front-right"}
    matches = []
    for module in modules:
        _check_deadline(deadline)
        if module["name"].startswith("libpipewire-module-"):
            continue
        try:
            with _DeadlineArgumentStream(module["argument"], deadline) as source:
                lexer = shlex.shlex(source, posix=True)
                lexer.whitespace_split = True
                lexer.commenters = ""
                arguments = []
                for argument in lexer:
                    _check_deadline(deadline)
                    arguments.append(argument)
        except EvidenceError:
            raise
        except ValueError as error:
            raise EvidenceError("malformed short module arguments") from error
        _check_deadline(deadline)
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
    _check_deadline(deadline)
    return matches


def owned_modules(host, name, *, deadline=None):
    """Return exact owned module records with their verified short-text index."""
    return _owned(_read_modules(host, deadline=deadline), name, deadline=deadline)


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


def require_sink_absent(host, name):
    """Refuse pre-load collisions without interpreting native module bodies."""
    if _sink_present(host.snapshot(), name):
        raise EvidenceError("proposed owned sink Node already exists")


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
        modules = _read_modules(host, deadline=deadline)
        _remaining(deadline)
        matches = _owned(modules, name, deadline=deadline)
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
