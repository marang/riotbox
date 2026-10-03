"""Child-local ALSA backend and coherent local PipeWire metadata endpoint.

ALSA_CONFIG_PATH replaces the global top-level config. This private config has
only PipeWire, no includes, hooks, aliases, hardware or fallback PCM. User/system
configuration is never rewritten. Effective route observation remains mandatory.
"""

from pathlib import Path

from silent_host_evidence import EvidenceError


def local_host_environment(parent):
    runtime = parent.get("XDG_RUNTIME_DIR", "")
    if not runtime or not Path(runtime).is_absolute() or any(c in runtime for c in "\n\r\0"):
        raise EvidenceError("absolute local user runtime directory is required")
    # Do not inherit alternate cores, plugins, backends, timing overrides or
    # injected libraries. Keep normal session/locale context without changing it.
    environment = {key: value for key, value in parent.items()
                   if not key.startswith(("ALSA_", "LIBASOUND_", "PIPEWIRE_", "PULSE_", "SPA_", "LD_"))}
    environment.update({"PIPEWIRE_REMOTE": "pipewire-0",
                        "PIPEWIRE_RUNTIME_DIR": runtime,
                        "PULSE_SERVER": f"unix:{runtime}/pulse/native"})
    return environment


def prepare_child_environment(owner, sink, parent):
    if type(sink.serial) is not int or sink.serial <= 0:
        raise EvidenceError("positive validated sink serial is required")
    path = owner / "pipewire-only-alsa.conf"
    if not path.is_absolute() or any(c in str(path) for c in ": \n\r\0"):
        raise EvidenceError("private ALSA config must be a single absolute path")
    environment = local_host_environment(parent)
    with path.open("x", encoding="utf-8") as config:
        config.write('pcm.!default {\n  type pipewire\n  server "pipewire-0"\n'
                     f'  playback_node "{sink.serial}"\n  channels 2\n}}\n')
    properties = (f"target.object={sink.serial} node.dont-fallback=true "
                  "node.dont-reconnect=true node.dont-move=true")
    environment.update({"ALSA_CONFIG_PATH": str(path),
                        "PIPEWIRE_NODE": str(sink.serial),
                        "PIPEWIRE_ALSA": properties, "PIPEWIRE_PROPS": properties})
    return environment
