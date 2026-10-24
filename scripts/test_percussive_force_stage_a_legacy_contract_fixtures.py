"""Public fixture recipes in generated metadata-only sandboxes; no real receipts."""

import os
from pathlib import Path
import py_compile
import shlex
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
RUNNER = "scripts/percussive_force_stage_a_legacy_contract_fixtures.py"
TRACKED_INPUTS = (
    "scripts/validate_percussive_force_stage_a_protocol_v3.py",
    "scripts/validate_percussive_force_stage_a_protocol_v4.py",
    "scripts/validate_percussive_force_stage_a_protocol_v5.py",
    "docs/benchmarks/percussive_force_stage_a_protocol_v2.json",
    "docs/benchmarks/percussive_force_stage_a_protocol_v3.json",
    "docs/benchmarks/percussive_force_stage_a_protocol_v4.json",
    "docs/benchmarks/percussive_force_stage_a_protocol_v5.json",
    "docs/benchmarks/percussive_force_stage_a_source_pool_v1.json",
    "docs/benchmarks/percussive_force_stage_a_source_pool_v2.json",
    "docs/benchmarks/percussive_force_development_matrix_template_v4.json",
    "docs/benchmarks/percussive_force_development_matrix_template_v5.json",
    "docs/benchmarks/source_holdout_rotation_v3.json",
)


class LegacyStageAContractFixtureTests(unittest.TestCase):
    def setUp(self):
        self.owner = tempfile.TemporaryDirectory(prefix="riotbox-legacy-stage-a-test-")
        self.addCleanup(self.owner.cleanup)
        self.sandbox = Path(self.owner.name)
        for relative in (*TRACKED_INPUTS, "Justfile", ".github/workflows/rust-ci.yml"):
            destination = self.sandbox / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, destination)
        if (ROOT / RUNNER).is_file():
            shutil.copyfile(ROOT / RUNNER, self.sandbox / RUNNER)

    def invoke(self, *args):
        environment = dict(os.environ)
        for key in ("PYTHONPATH", "PYTHONHOME", "ENV", "BASH_ENV"):
            environment.pop(key, None)
        return subprocess.run(args, cwd=self.sandbox, env=environment, capture_output=True,
                              text=True, check=False, timeout=30)

    def recipe(self, version):
        name = f"percussive-force-stage-a-{version}-contract-fixtures"
        if shutil.which("just"):
            return self.invoke("just", "--justfile", str(self.sandbox / "Justfile"), name)
        # GitHub's Python-check job lacks Just: run this same public recipe's CLI.
        text = (self.sandbox / "Justfile").read_text()
        lines = text.split(f"\n{name}:\n", 1)[1].split("\n\n", 1)[0].splitlines()
        self.assertEqual(len(lines), 1)
        argv = shlex.split(lines[0])
        self.assertEqual(argv[0], "python3")
        return self.invoke(sys.executable, *argv[1:])

    def test_v4_public_fixture_recipe_needs_no_historical_receipts(self):
        result = self.recipe("v4")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name in ("algorithm", "fmt", "resume", "admitted", "retry_float", "matrix"):
            self.assertIn(f"PASS mutation {name}\n", result.stdout)
        self.assertIn("historical_receipts_verified=false", result.stdout)
        self.assertFalse((self.sandbox / "artifacts").exists())

    def test_v5_public_fixture_recipe_needs_neither_historical_receipt(self):
        result = self.recipe("v5")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name in ("request", "retry", "algorithm", "matrix"):
            self.assertIn(f"PASS mutation {name}\n", result.stdout)
        self.assertIn("historical_receipts_verified=false", result.stdout)
        self.assertFalse((self.sandbox / "artifacts").exists())

    def test_v4_terminal_access_negative_controls_run(self):
        result = self.recipe("v4")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name in ("state", "attempted", "completed", "failure", "pcm", "playback", "holdout", "commercial"):
            self.assertIn(f"PASS terminal mutation v3_{name}\n", result.stdout)

    def test_v5_terminal_access_negative_controls_run(self):
        result = self.recipe("v5")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for name in ("state", "attempted", "completed", "failure", "ordinal", "entry_state", "pcm", "playback", "holdout"):
            self.assertIn(f"PASS terminal mutation v4_{name}\n", result.stdout)

    def test_every_tracked_input_raw_pin_is_enforced_before_validator_execution(self):
        for relative in TRACKED_INPUTS:
            path = self.sandbox / relative
            original = path.read_bytes()
            for version in ("v4", "v5"):
                with self.subTest(path=relative, version=version):
                    extra = b'\nraise RuntimeError("UNPINNED_CODE_EXECUTED")\n' if relative.endswith(".py") else b"\n"
                    path.write_bytes(original + extra)
                    result = self.recipe(version)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(f"tracked fixture input pin changed: {relative}", result.stdout)
                    self.assertNotIn("UNPINNED_CODE_EXECUTED", result.stderr)
            path.write_bytes(original)
        self.assertFalse((self.sandbox / "artifacts").exists())

    def test_missing_or_linked_tracked_metadata_is_not_substituted(self):
        path = self.sandbox / "docs/benchmarks/percussive_force_stage_a_protocol_v4.json"
        original = path.read_bytes()
        path.unlink()
        result = self.recipe("v4")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tracked fixture input is not a regular unlinked path", result.stdout)
        target = self.sandbox / "generated-sentinel.json"
        target.write_bytes(original)
        path.symlink_to(target)
        result = self.recipe("v5")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tracked fixture input is not a regular unlinked path", result.stdout)

    def test_pinned_source_is_executed_instead_of_untracked_python_bytecode(self):
        path = self.sandbox / "scripts/validate_percussive_force_stage_a_protocol_v3.py"
        original = path.read_bytes()
        identity = path.stat()
        poison = b'raise RuntimeError("UNPINNED_CACHE_EXECUTED")\n'
        path.write_bytes(poison + b"#" * (len(original) - len(poison)))
        os.utime(path, ns=(identity.st_atime_ns, identity.st_mtime_ns))
        py_compile.compile(str(path), doraise=True)
        path.write_bytes(original)
        os.utime(path, ns=(identity.st_atime_ns, identity.st_mtime_ns))
        result = self.recipe("v4")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("UNPINNED_CACHE_EXECUTED", result.stderr)

    def test_production_commands_still_reject_missing_and_mismatched_historical_receipts(self):
        for version, receipt_version, reason in ((4, 3, "v4 raw document pin changed"),
                                                  (5, 4, "v4 access raw SHA changed")):
            with self.subTest(version=version):
                command = (sys.executable, f"scripts/validate_percussive_force_stage_a_protocol_v{version}.py", "--fixtures")
                missing = self.invoke(*command)
                self.assertNotEqual(missing.returncode, 0)
                self.assertIn(f"stage-a-v{receipt_version}-source-pool-access.json", missing.stdout)
                self.assertIn("No such file", missing.stdout)
                # Deliberately invalid bytes only in this owned generated sandbox.
                # Never create/replace a real receipt or claim historical validity.
                invalid_path = self.sandbox / f"artifacts/audio_qa/riotbox-1430/stage-a-v{receipt_version}-source-pool-access.json"
                invalid_path.parent.mkdir(parents=True, exist_ok=True)
                invalid_path.write_text('{"deliberately_invalid_sandbox_negative": true}\n')
                mismatch = self.invoke(*command)
                self.assertNotEqual(mismatch.returncode, 0)
                self.assertIn(reason, mismatch.stdout)
                self.assertNotIn("PASS mutation", mismatch.stdout)

    def test_synthetic_recipes_do_not_consume_or_rewrite_invalid_sandbox_receipts(self):
        invalid = b"not historical evidence, deliberately not JSON\n"
        for version in (3, 4):
            path = self.sandbox / f"artifacts/audio_qa/riotbox-1430/stage-a-v{version}-source-pool-access.json"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(invalid)
        for version in ("v4", "v5"):
            result = self.recipe(version)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("historical_receipts_verified=false", result.stdout)
        for version in (3, 4):
            path = self.sandbox / f"artifacts/audio_qa/riotbox-1430/stage-a-v{version}-source-pool-access.json"
            self.assertEqual(path.read_bytes(), invalid)

    def test_regression_gate_is_wired_in_source_free_ci_and_github_python_checks(self):
        recipes = (self.sandbox / "Justfile").read_text()
        ci = recipes.split("\nci:\n", 1)[1].split("\n\n", 1)[0]
        self.assertIn("just percussive-force-stage-a-legacy-fixture-regressions", ci)
        command = "python3 -m unittest discover -s scripts -p 'test_percussive_force_stage_a_legacy_contract_fixtures.py'"
        self.assertIn(command, recipes)
        self.assertIn(command, (self.sandbox / ".github/workflows/rust-ci.yml").read_text())


if __name__ == "__main__":
    unittest.main()
