"""Generated-only source selector compatibility; no runtime or real audio."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import wave


GENERATOR = Path(__file__).resolve().with_name("write_diverse_test_source_wavs.py")
CASES = (
    "dense_break_132", "sparse_kick_snare_118", "tonal_hook_124", "pad_noise_96",
    "sub_pressure_140", "broken_timing_127", "syncopated_break_136",
    "offgrid_sparse_122", "acid_riff_126", "stab_hit_128",
    "stop_start_break_140", "thin_weak_loop_110",
)


def generate(output, *extra):
    return subprocess.run(
        [sys.executable, str(GENERATOR), "--output", str(output), "--seconds", "0.02", *extra],
        capture_output=True, check=False, timeout=30,
    )


class GeneratedLoadSourceTests(unittest.TestCase):
    def test_default_corpus_and_selected_source_are_byte_identical(self):
        with tempfile.TemporaryDirectory() as owner:
            root = Path(owner)
            all_sources = root / "default"
            selected = root / "selected"
            self.assertEqual(generate(all_sources).returncode, 0)
            self.assertEqual(generate(selected, "--only", "dense_break_132").returncode, 0)
            default = json.loads((all_sources / "manifest.json").read_text())
            single = json.loads((selected / "manifest.json").read_text())
            self.assertEqual(default["case_count"], 12)
            self.assertEqual(tuple(entry["case_id"] for entry in default["entries"]), CASES)
            self.assertEqual(single["case_count"], 1)
            self.assertEqual(single["entries"][0]["case_id"], "dense_break_132")
            self.assertEqual({p.name for p in selected.iterdir()}, {"manifest.json", "dense_break_132.wav"})
            self.assertEqual(
                hashlib.sha256((all_sources / "dense_break_132.wav").read_bytes()).digest(),
                hashlib.sha256((selected / "dense_break_132.wav").read_bytes()).digest(),
            )
            for manifest in (default, single):
                self.assertEqual(manifest["schema"], "riotbox.diverse_test_source_wavs.v1")
                self.assertTrue(all(entry["quality_proof"] is False for entry in manifest["entries"]))
            with wave.open(str(selected / "dense_break_132.wav"), "rb") as audio:
                self.assertEqual((audio.getframerate(), audio.getnchannels(), audio.getsampwidth(), audio.getnframes()),
                                 (44100, 2, 2, 882))

    def test_selection_is_reproducible_and_unknown_case_creates_nothing(self):
        with tempfile.TemporaryDirectory() as owner:
            root = Path(owner)
            for name in ("first", "second"):
                self.assertEqual(generate(root / name, "--only", "dense_break_132").returncode, 0)
            self.assertEqual((root / "first/dense_break_132.wav").read_bytes(),
                             (root / "second/dense_break_132.wav").read_bytes())
            invalid = root / "invalid"
            self.assertNotEqual(generate(invalid, "--only", "unregistered").returncode, 0)
            self.assertFalse(invalid.exists())


if __name__ == "__main__":
    unittest.main()
