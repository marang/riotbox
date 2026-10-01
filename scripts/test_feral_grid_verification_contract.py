"""Source-free Feral grid recipe quoting checks; real Just is optional in CI."""

from pathlib import Path
import os
import re
import shutil
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[1]


class FeralGridVerificationContractTests(unittest.TestCase):
    def test_every_recipe_interpolation_is_bourne_shell_quoted(self):
        text = (ROOT / "Justfile").read_text()
        recipe = text.split("feral-grid-pack source ", 1)[1].split("\n\n", 1)[0]
        interpolations = re.findall(r"\{\{(.*?)\}\}", recipe)
        expected = {"source": 2, "date": 2, "bpm": 2, "bars": 2,
                    "source_window": 2, "start": 2}
        self.assertEqual(len(interpolations), sum(expected.values()))
        for name, count in expected.items():
            self.assertEqual(interpolations.count(f"quote({name})"), count, name)
        self.assertIn('if [ {{quote(bpm)}} = "auto" ]; then', recipe)
        self.assertNotRegex(recipe, r'"\{\{quote\(')

    @unittest.skipUnless(shutil.which("just") and os.name == "posix",
                         "actual Just recipe probe requires local Just and POSIX sh")
    def test_actual_just_recipe_preserves_literal_cargo_argv(self):
        # Only the dry-run script is evaluated, with an argv-only Cargo function.
        # Never execute a generated command, the renderer, or a filesystem payload.
        values = ["plain", "space tab\tline\nend", "single'and\"double", "back\\slash",
                  "$RBX_QUOTE_PROBE", "$(printf expanded)", "`printf expanded`",
                  "semi;colon", "--option-like", "Grüße 🎛", "", "source=value"]
        cases = [(value, value, bpm, "2", "0.750", "0.125")
                 for value in values for bpm in ("auto", "120.125")]
        for index in range(2, 6):
            case = ["control.wav", "date", "auto", "2", "0.750", "0.125"]
            case[index] = "$(printf expanded) ' \""
            cases.append(tuple(case))
        for source, date, bpm, bars, window, start in cases:
            with self.subTest(source=source, date=date, bpm=bpm):
                dry = subprocess.run(
                    ["just", "--dry-run", "feral-grid-pack", source, date,
                     bpm, bars, window, start], cwd=ROOT, capture_output=True, check=True)
                self.assertFalse(dry.stdout)
                self.assertTrue(dry.stderr.startswith(b"if [ "))
                env = dict(os.environ, RBX_QUOTE_PROBE="expanded")
                env.pop("ENV", None)
                env.pop("BASH_ENV", None)
                probe = subprocess.run(
                    ["sh", "-c", "cargo() { printf '%s\\000' \"$@\"; }\n" + dry.stderr.decode()],
                    capture_output=True, env=env, check=True)
                self.assertFalse(probe.stderr)
                expected = ["run", "-p", "riotbox-audio", "--bin", "feral_grid_pack", "--",
                            "--source", source, "--date", date]
                if bpm != "auto":
                    expected += ["--bpm", bpm]
                expected += ["--bars", bars, "--source-window-seconds", window,
                             "--source-start-seconds", start, ""]
                self.assertEqual(probe.stdout.split(b"\0"), [arg.encode() for arg in expected])


if __name__ == "__main__":
    unittest.main()
