"""Pure, fail-closed PipeWire lifetime admission for Silent Host Observation V2."""

from dataclasses import dataclass


class RouteError(ValueError):
    pass


@dataclass(frozen=True)
class SinkIdentity:
    node_id: int
    serial: int
    name: str


@dataclass(frozen=True)
class NodeIdentity:
    node_id: int
    serial: int

    def __post_init__(self):
        if any(type(value) is not int or value < 0
               for value in (self.node_id, self.serial)):
            raise RouteError("node identity requires nonnegative integer ID and serial")
        if self.node_id >= 2**32 or self.serial >= 2**64:
            raise RouteError("node identity exceeds the PipeWire ID/serial integer domain")

    @classmethod
    def from_node(cls, item):
        if not kind(item, "Node"):
            raise RouteError("stream identity must name a Node")
        return cls(number(item["id"]), number(props(item).get("object.serial")))


@dataclass(frozen=True)
class RouteIdentity:
    client_id: int
    node: NodeIdentity
    link_ids: tuple[int, int]


def number(value):
    if isinstance(value, bool) or not str(value).isdecimal():
        raise RouteError(f"invalid object identifier: {value!r}")
    return int(value)


def indexed(snapshot):
    if not isinstance(snapshot, list):
        raise RouteError("snapshot must be an object list")
    objects = {}
    for item in snapshot:
        if not isinstance(item, dict) or "id" not in item:
            raise RouteError("invalid snapshot object")
        object_id = number(item["id"])
        if object_id in objects:
            raise RouteError("duplicate snapshot object ID")
        if item.get("type") in {f"PipeWire:Interface:{k}"
                                for k in ("Node", "Client", "Port", "Link")}:
            if not isinstance(item.get("info"), dict):
                raise RouteError("observed object info must be a map")
        objects[object_id] = item
    return objects


def kind(item, expected):
    return item.get("type") == f"PipeWire:Interface:{expected}"


def props(item):
    value = item.get("info", {}).get("props", {})
    if not isinstance(value, dict):
        raise RouteError("object properties must be a map")
    return value


def find_sink(snapshot, name):
    matches = [item for item in indexed(snapshot).values()
               if kind(item, "Node") and props(item).get("node.name") == name]
    if len(matches) != 1 or props(matches[0]).get("media.class") != "Audio/Sink":
        raise RouteError("exact owned sink is missing or ambiguous")
    return SinkIdentity(number(matches[0]["id"]),
                        number(props(matches[0]).get("object.serial")), name)


def attached_streams(snapshot, sink):
    """Even a temporarily unlinked stream keeps the containment sink alive."""
    return [number(item["id"]) for item in indexed(snapshot).values()
            if kind(item, "Node") and props(item).get("media.class") == "Stream/Output/Audio"
            and str(props(item).get("target.object")) == str(sink.serial)]


def node_present(snapshot, identity):
    """A global ID may be recycled; only the original typed lifetime survives."""
    objects = indexed(snapshot)
    for candidate in objects.values():
        if kind(candidate, "Node"):
            current = NodeIdentity.from_node(candidate)
            if current.serial == identity.serial and current.node_id != identity.node_id:
                raise RouteError("original node serial has inconsistent global identity")
    item = objects.get(identity.node_id)
    if item is None:
        return False
    interface = item.get("type")
    if (not isinstance(interface, str) or not interface.startswith("PipeWire:Interface:")
            or not interface.removeprefix("PipeWire:Interface:")):
        raise RouteError("node lifetime observation has no valid object type")
    if not kind(item, "Node"):
        return False
    return NodeIdentity.from_node(item) == identity


def observe_route(snapshot, sink, pid):
    """Return exact active route IDs, None for safe pending links, or raise.

    Sink identity and absence of outgoing/foreign links are checked even while
    the process is starting or exiting. Callers must bound the None/startup state
    and reject it during an admitted run, except its evidenced natural teardown.
    """
    objects = indexed(snapshot)
    if find_sink(snapshot, sink.name) != sink:
        raise RouteError("owned sink identity changed")
    links = [item for item in objects.values() if kind(item, "Link")]
    for link in links:
        if number(link["info"].get("output-node-id")) == sink.node_id:
            raise RouteError("owned sink has an outgoing link")

    clients = [item for item in objects.values()
               if kind(item, "Client")
               and str(props(item).get("application.process.id")) == str(pid)]
    client_ids = {number(item["id"]) for item in clients}
    nodes = [item for item in objects.values()
             if kind(item, "Node")
             and props(item).get("client.id") is not None
             and number(props(item)["client.id"]) in client_ids]
    incoming = [link for link in links
                if number(link["info"].get("input-node-id")) == sink.node_id]
    if not nodes:
        if incoming:
            raise RouteError("owned sink has foreign incoming links")
        return None
    if len(clients) != 1 or len(nodes) != 1:
        raise RouteError("process stream is ambiguous")
    node = nodes[0]
    node_id = number(node["id"])
    node_props = props(node)
    if node_props.get("media.class") != "Stream/Output/Audio":
        raise RouteError("process node is not an output audio stream")
    if str(node_props.get("target.object")) != str(sink.serial):
        raise RouteError("process target does not name the owned sink serial")
    for key in ("node.dont-fallback", "node.dont-reconnect", "node.dont-move"):
        if not (node_props.get(key) is True or node_props.get(key) == "true"):
            raise RouteError(f"required safety property missing: {key}")
    outgoing = [link for link in links
                if number(link["info"].get("output-node-id")) == node_id]
    if any(number(link["info"].get("output-node-id")) != node_id for link in incoming):
        raise RouteError("owned sink has foreign incoming links")
    if any(number(link["info"].get("input-node-id")) == node_id for link in links):
        raise RouteError("output stream has incoming links")
    if len(outgoing) > 2 or len(incoming) > 2:
        raise RouteError("more than the owned stereo link pair is present")
    channels = set()
    all_active = True
    for link in outgoing:
        info = link["info"]
        if number(info["input-node-id"]) != sink.node_id:
            raise RouteError("stream link targets another sink")
        state = info.get("state")
        if state not in ("active", "init", "negotiating", "allocating", "paused"):
            raise RouteError("stream link state is failed or unknown")
        all_active = all_active and state == "active"
        output_port = objects.get(number(info.get("output-port-id")), {})
        input_port = objects.get(number(info.get("input-port-id")), {})
        if not kind(output_port, "Port") or not kind(input_port, "Port"):
            raise RouteError("stream link port is absent")
        channel = props(output_port).get("audio.channel")
        if (number(props(output_port).get("node.id")) != node_id
                or number(props(input_port).get("node.id")) != sink.node_id
                or channel != props(input_port).get("audio.channel")
                or channel not in ("FL", "FR")):
            raise RouteError("stream link does not preserve owned stereo channels")
        if channel in channels:
            raise RouteError("stereo channel appears twice")
        channels.add(channel)
    if channels != {"FL", "FR"} or not all_active:
        return None
    return RouteIdentity(number(clients[0]["id"]), NodeIdentity.from_node(node),
                         tuple(sorted(number(link["id"]) for link in outgoing)))
