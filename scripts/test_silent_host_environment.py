"""Generated configuration/environment checks, never an ALSA PCM open."""

import os
from pathlib import Path
import tempfile
import unittest

from silent_host_environment import local_host_environment, prepare_child_environment
from silent_host_evidence import EvidenceError
from silent_host_routes import SinkIdentity


class EnvironmentTests(unittest.TestCase):
    def test_host_and_child_share_local_endpoint_and_inherited_overrides_are_removed(self):
        original = {"XDG_RUNTIME_DIR": "/run/user/1000", "HOME": "/generated-home",
                    "PATH": os.defpath, "ALSA_CONFIG_PATH": "/unsafe-hw-config",
                    "ALSA_PLUGIN_DIR": "/unexpected-plugin",
                    "PIPEWIRE_PROPS": "target.object=physical node.dont-fallback=false",
                    "PIPEWIRE_ALSA": "target.object=physical", "PIPEWIRE_NODE": "physical",
                    "PIPEWIRE_REMOTE": "other-core", "PIPEWIRE_RUNTIME_DIR": "/other-runtime",
                    "PIPEWIRE_CONFIG_DIR": "/unexpected-config", "PIPEWIRE_RATE": "96000",
                    "SPA_PLUGIN_DIR": "/unexpected-spa", "LD_PRELOAD": "/unexpected.so",
                    "PULSE_SERVER": "tcp:somewhere", "LIBASOUND_THREAD_SAFE": "0"}
        host = local_host_environment(original)
        self.assertEqual(original["ALSA_CONFIG_PATH"], "/unsafe-hw-config")
        self.assertEqual(host["HOME"], original["HOME"])
        self.assertEqual(host["PIPEWIRE_REMOTE"], "pipewire-0")
        self.assertEqual(host["PIPEWIRE_RUNTIME_DIR"], "/run/user/1000")
        self.assertEqual(host["PULSE_SERVER"], "unix:/run/user/1000/pulse/native")
        for key in original.keys() - {"HOME", "PATH", "XDG_RUNTIME_DIR",
                                       "PIPEWIRE_REMOTE", "PIPEWIRE_RUNTIME_DIR", "PULSE_SERVER"}:
            self.assertNotIn(key, host)
        with tempfile.TemporaryDirectory(prefix="riotbox-alsa-config-") as directory:
            owner = Path(directory)
            child = prepare_child_environment(owner, SinkIdentity(10, 1000, "owned-test"), host)
            config = Path(child["ALSA_CONFIG_PATH"])
            self.assertEqual(config.parent, owner)
            self.assertEqual(config.read_text(),
                             'pcm.!default {\n  type pipewire\n  server "pipewire-0"\n'
                             '  playback_node "1000"\n  channels 2\n}\n')
            self.assertEqual(child["PIPEWIRE_NODE"], "1000")
            self.assertEqual(child["PIPEWIRE_PROPS"], child["PIPEWIRE_ALSA"])
            for value in ("target.object=1000", "node.dont-fallback=true",
                          "node.dont-reconnect=true", "node.dont-move=true"):
                self.assertIn(value, child["PIPEWIRE_PROPS"].split())
            self.assertEqual(child["PIPEWIRE_REMOTE"], host["PIPEWIRE_REMOTE"])
            with self.assertRaises(FileExistsError):
                prepare_child_environment(owner, SinkIdentity(10, 1000, "owned-test"), host)

    def test_missing_or_relative_runtime_context_is_rejected(self):
        for value in (None, "", "relative", "/run/user/1000\nother"):
            with self.subTest(value=value), self.assertRaises(EvidenceError):
                local_host_environment({} if value is None else {"XDG_RUNTIME_DIR": value})

    def test_unvalidated_serial_cannot_inject_alsa_configuration(self):
        for value in (0, -1, True, '1000" }\npcm.hw { type hw }'):
            with self.subTest(serial=value), tempfile.TemporaryDirectory(prefix="riotbox-alsa-invalid-") as directory:
                owner = Path(directory)
                with self.assertRaises(EvidenceError):
                    prepare_child_environment(owner, SinkIdentity(10, value, "owned-test"),
                                              {"XDG_RUNTIME_DIR": "/run/user/1000"})
                self.assertEqual(list(owner.iterdir()), [])

    def test_alsa_file_list_separators_are_not_admitted_in_private_path(self):
        for path in (Path("relative"), Path("/generated:other"), Path("/generated space")):
            with self.subTest(path=path), self.assertRaises(EvidenceError):
                prepare_child_environment(path, SinkIdentity(10, 1000, "owned-test"),
                                          {"XDG_RUNTIME_DIR": "/run/user/1000"})


if __name__ == "__main__":
    unittest.main()
