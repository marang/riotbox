#!/usr/bin/env python3
"""Synthetic-only tests: never open registered/protected audio or a device."""

from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import stage_source_evaluation_development as staging


class DevelopmentStagingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="riotbox-development-staging-test-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self.base / "source"
        self.target = self.base / "validation"
        self.source.mkdir()
        self.target.mkdir()
        (self.target / ".git").write_text("gitdir: synthetic-test-only\n")
        self.receipt = self.base / "receipt.json"
        self.payloads = [f"not audio; synthetic Development source {index}\n".encode()
                         for index in range(7)]
        self.policy = {
            "schema": "riotbox.source_evaluation_alpha_development_access.v1",
            "owner_ticket": "RIOTBOX-1609", "authorization_date": "2026-10-24",
            "broad_invocation_limit": 1, "original_reads_per_source_limit": 1,
            "maximum_source_bytes": 1024, "quality_proof": False, "playback_allowed": False,
            "commercial_reference_material_allowed": False,
            "protected_registries": list(staging.REGISTRY_PATHS),
            "rights_context": "Synthetic generated data; no source audio.",
            "sources": [],
        }
        for index, (case_id, name, enrollment) in enumerate(staging.EXACT_SOURCES):
            relative = f"data/test_audio/examples/{name}"
            self.policy["sources"].append({
                "case_id": case_id, "path": relative,
                "historical_sha256": None if enrollment else staging.sha256(self.payloads[index]),
                "allow_hash_enrollment": enrollment, "identity_record": "synthetic metadata",
            })
            self.write(self.source / relative, self.payloads[index])
        self.registries = [{
            "schema": f"riotbox.source_holdout_rotation.v{version}", "schema_version": version,
            "commercial_reference_material_allowed": False,
            "entries": [{"case_id": f"synthetic_protected_{version}", "partition": "holdout_a",
                         "source_path": f"never/open/protected-{version}.wav",
                         "sha256": staging.sha256(f"never-open-protected-{version}".encode()),
                         "commercial_reference": False}],
        } for version in (1, 2, 3)]
        self.refresh_metadata()

    @staticmethod
    def write(path: Path, payload: bytes) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)

    def refresh_metadata(self) -> None:
        self.policy_payload = json.dumps(self.policy).encode()
        self.write(self.source / staging.POLICY_PATH, self.policy_payload)
        self.pins = {}
        for relative, registry in zip(staging.REGISTRY_PATHS, self.registries):
            payload = json.dumps(registry).encode()
            self.write(self.source / relative, payload)
            self.pins[relative] = staging.sha256(payload)

    def run_stage(self, *, acknowledgement: bool = True) -> dict:
        return staging.stage(self.source, self.target, self.receipt,
                             staging.sha256(self.policy_payload), self.pins,
                             acknowledge_new_hash_enrollment=acknowledgement)

    def failure(self, reason: str, *, acknowledgement: bool = True) -> dict:
        with self.assertRaisesRegex((ValueError, OSError), reason):
            self.run_stage(acknowledgement=acknowledgement)
        receipt = json.loads(self.receipt.read_text())
        self.assertEqual(receipt["status"], "failed_stop_no_retry")
        self.assertEqual(receipt["broad_invocations"], 0)
        return receipt

    def assert_no_source_opens(self, receipt: dict) -> None:
        self.assertEqual(receipt["sources"], [])
        self.assertEqual(receipt["staged_paths"], [])

    def test_seven_success_exact_single_reads_and_honest_enrollment(self) -> None:
        # Pin arguments need not arrive in version order.
        self.pins = dict(reversed(list(self.pins.items())))
        receipt = self.run_stage()
        self.assertTrue(receipt["status"].startswith("staged_ready"))
        self.assertEqual(len(receipt["frozen_identities"]), 7)
        self.assertEqual(len(receipt["metadata"]), 4)
        for index, source in enumerate(receipt["sources"]):
            self.assertEqual((self.target / source["path"]).read_bytes(), self.payloads[index])
            events = [event["event"] for event in source["events"]]
            self.assertEqual(events.count("opened"), 1)
            self.assertEqual(events.count("bytes_read"), 1)
            self.assertEqual(events.count("sha256_computed"), 1)
            self.assertEqual(source["hash_basis"], "authorized_new_enrollment" if index in (1, 3)
                             else "historical_expected_sha256")
            if index in (1, 3):
                self.assertIsNone(source["historical_sha256"])
        self.assertNotIn("protected_audio_accessed", receipt)
        self.assertFalse(receipt["protected_registered_paths_opened"])
        self.assertFalse(receipt["protected_hash_collision_observed"])
        self.assertEqual(receipt["protected_hash_identity_assessment"], "all_seven_no_collision")
        self.assertFalse(receipt["audio_decoded"])
        self.assertEqual(receipt["broad_invocations"], 0)
        self.assertNotIn("not audio;", self.receipt.read_text())

    def test_all_registry_versions_protect_before_any_original_stat(self) -> None:
        original = copy.deepcopy(self.registries)
        for version in range(3):
            for field, expected in (("case_id", "protected case"), ("source_path", "protected path"),
                                    ("sha256", "protected expected SHA")):
                with self.subTest(version=version + 1, field=field):
                    self.registries = copy.deepcopy(original)
                    source_key = {"case_id": "case_id", "source_path": "path",
                                  "sha256": "historical_sha256"}[field]
                    self.registries[version]["entries"][0][field] = self.policy["sources"][0][source_key]
                    self.refresh_metadata()
                    receipt = self.failure(expected)
                    self.assert_no_source_opens(receipt)
                    self.receipt.unlink()

    def test_observed_source_opens_are_exact_once_without_discovery(self) -> None:
        opened: list[str] = []
        original_open = os.open

        def record_open(path, flags, *args, **kwargs):
            opened.append(str(path))
            return original_open(path, flags, *args, **kwargs)

        with patch.object(staging.os, "open", side_effect=record_open), \
                patch.object(staging.os, "listdir", side_effect=AssertionError("directory discovery")), \
                patch.object(staging.os, "scandir", side_effect=AssertionError("directory discovery")):
            self.run_stage()
        for _, filename, _ in staging.EXACT_SOURCES:
            # One original read open plus one exclusive staging write open.
            self.assertEqual(opened.count(filename), 2)
        self.assertFalse(any("protected-" in path for path in opened))

    def test_commercial_development_identity_is_protected(self) -> None:
        entry = self.registries[2]["entries"][0]
        entry.update(partition="development", commercial_reference=True,
                     case_id=self.policy["sources"][0]["case_id"])
        self.refresh_metadata()
        self.assert_no_source_opens(self.failure("protected case"))

    def test_non_development_partition_is_protected(self) -> None:
        self.registries[1]["entries"][0].update(
            partition="consumed", source_path=self.policy["sources"][0]["path"])
        self.refresh_metadata()
        self.assert_no_source_opens(self.failure("protected path"))

    def test_enrolled_hash_collision_stops_before_staging_and_next_open(self) -> None:
        self.registries[2]["entries"][0]["sha256"] = staging.sha256(self.payloads[1])
        self.refresh_metadata()
        receipt = self.failure("observed protected SHA")
        self.assertEqual(len(receipt["sources"]), 2)
        self.assertEqual(receipt["sources"][-1]["observed_sha256"], staging.sha256(self.payloads[1]))
        self.assertEqual(receipt["sources"][-1]["status"], "pending")
        self.assertEqual(receipt["staged_paths"], [])
        self.assertEqual(receipt["frozen_identities"], [])
        self.assertNotIn("protected_audio_accessed", receipt)
        self.assertFalse(receipt["protected_registered_paths_opened"])
        self.assertTrue(receipt["protected_hash_collision_observed"])
        self.assertEqual(receipt["protected_hash_identity_assessment"], "collision_observed")

    def test_nested_available_protected_hash_is_excluded(self) -> None:
        self.registries[1]["entries"][0]["conversion"] = {
            "original_sha256": self.policy["sources"][0]["historical_sha256"]}
        self.refresh_metadata()
        self.assert_no_source_opens(self.failure("protected expected SHA"))

    def test_missing_enrollment_acknowledgement_opens_no_sources(self) -> None:
        receipt = self.failure("acknowledgement", acknowledgement=False)
        self.assert_no_source_opens(receipt)

    def test_unknown_schema_limit_claim_and_field_changes_rejected(self) -> None:
        original = copy.deepcopy(self.policy)
        changes = [("schema", "unknown"), ("broad_invocation_limit", 2),
                   ("original_reads_per_source_limit", 2), ("maximum_source_bytes", 67_108_865),
                   ("playback_allowed", True), ("quality_proof", True),
                   ("commercial_reference_material_allowed", True), ("extra", False)]
        for key, value in changes:
            with self.subTest(key=key):
                self.policy = copy.deepcopy(original)
                self.policy[key] = value
                self.refresh_metadata()
                self.assert_no_source_opens(self.failure("policy|source byte|unknown"))
                self.receipt.unlink()

    def test_unknown_duplicate_and_traversal_source_paths_rejected(self) -> None:
        original = copy.deepcopy(self.policy)
        changes = [("path", "data/../secret.wav"), ("path", "/absolute.wav"),
                   ("path", "data//file.wav"), ("case_id", "unregistered"),
                   ("path", original["sources"][1]["path"]), ("allow_hash_enrollment", True)]
        for key, value in changes:
            with self.subTest(key=key, value=value):
                self.policy = copy.deepcopy(original)
                self.policy["sources"][0][key] = value
                self.refresh_metadata()
                self.assert_no_source_opens(self.failure("path|identity|enrollment"))
                self.receipt.unlink()

    def test_metadata_pin_change_rejected_before_sources(self) -> None:
        self.pins[staging.REGISTRY_PATHS[-1]] = "0" * 64
        self.assert_no_source_opens(self.failure("metadata pin mismatch"))

    def test_duplicate_metadata_key_rejected_before_sources(self) -> None:
        self.policy_payload = b'{"schema":"first","schema":"second"}'
        self.write(self.source / staging.POLICY_PATH, self.policy_payload)
        self.assert_no_source_opens(self.failure("duplicate JSON"))

    def test_symlink_source_refused_before_open(self) -> None:
        path = self.source / self.policy["sources"][0]["path"]
        path.unlink()
        path.symlink_to(self.source / self.policy["sources"][1]["path"])
        receipt = self.failure("regular, not a symlink")
        self.assertEqual(len(receipt["sources"]), 1)
        self.assertNotIn("open_attempted", [event["event"] for event in receipt["sources"][0]["events"]])

    def test_symlink_source_directory_refused(self) -> None:
        parent = self.source / "data/test_audio"
        moved = self.source / "moved"
        parent.rename(moved)
        parent.symlink_to(moved, target_is_directory=True)
        receipt = self.failure("Not a directory|Too many levels")
        self.assertEqual(receipt["sources"][0]["events"], [])

    def test_hardlink_refused_before_open(self) -> None:
        os.link(self.source / self.policy["sources"][0]["path"], self.base / "extra-link")
        receipt = self.failure("one hard link")
        self.assertEqual(receipt["sources"][0]["events"][-1]["event"], "stat_observed")

    def test_fifo_refused_without_blocking_or_opening(self) -> None:
        path = self.source / self.policy["sources"][0]["path"]
        path.unlink()
        os.mkfifo(path)
        receipt = self.failure("regular")
        self.assertEqual(receipt["sources"][0]["events"][-1]["event"], "stat_observed")

    def test_oversized_refused_before_open(self) -> None:
        self.write(self.source / self.policy["sources"][0]["path"], b"x" * 1025)
        receipt = self.failure("pre-open byte bound")
        self.assertEqual(receipt["sources"][0]["events"][-1]["event"], "stat_observed")

    def test_changed_bytes_refuse_hash_and_stop_no_later_sources(self) -> None:
        self.write(self.source / self.policy["sources"][2]["path"], b"different bytes")
        receipt = self.failure("historical expected SHA mismatch")
        self.assertEqual(len(receipt["sources"]), 3)
        self.assertEqual(receipt["staged_paths"], [])

    def test_mutation_during_read_is_rejected_before_hash(self) -> None:
        original_read = os.read
        source_path = self.source / self.policy["sources"][0]["path"]

        def mutate(fd: int, amount: int) -> bytes:
            payload = original_read(fd, amount)
            if payload == self.payloads[0]:
                source_path.write_bytes(b"changed after read")
            return payload

        with patch.object(staging.os, "read", side_effect=mutate):
            receipt = self.failure("changed during bounded read")
        self.assertEqual(len(receipt["sources"]), 1)
        self.assertNotIn("observed_sha256", receipt["sources"][0])

    def test_destination_no_overwrite_before_original_access(self) -> None:
        path = self.target / self.policy["sources"][3]["path"]
        self.write(path, b"existing user output")
        self.assert_no_source_opens(self.failure("no overwrite"))
        self.assertEqual(path.read_bytes(), b"existing user output")

    def test_receipt_no_overwrite_and_no_retry(self) -> None:
        self.receipt.write_text("existing evidence")
        with self.assertRaises(FileExistsError):
            self.run_stage()
        self.assertEqual(self.receipt.read_text(), "existing evidence")

    def test_destination_symlink_parent_and_existing_qa_preserved(self) -> None:
        qa = self.target / "qa/existing.txt"
        self.write(qa, b"keep existing QA")
        outside = self.base / "outside"
        outside.mkdir()
        (self.target / "data").symlink_to(outside, target_is_directory=True)
        self.assert_no_source_opens(self.failure("Not a directory|Too many levels"))
        self.assertEqual(qa.read_bytes(), b"keep existing QA")
        self.assertEqual(list(outside.iterdir()), [])

    def test_output_race_does_not_overwrite_and_keeps_frozen_receipt(self) -> None:
        original_open = os.open

        def output_race(path: str, flags: int, *args, **kwargs) -> int:
            if str(path) == staging.EXACT_SOURCES[0][1] and flags & os.O_CREAT:
                competing_fd = original_open(path, flags, *args, **kwargs)
                os.write(competing_fd, b"competing output")
                os.close(competing_fd)
            return original_open(path, flags, *args, **kwargs)

        with patch.object(staging.os, "open", side_effect=output_race):
            receipt = self.failure("File exists")
        self.assertEqual(len(receipt["frozen_identities"]), 7)
        self.assertEqual(receipt["staged_paths"], [])
        self.assertEqual((self.target / self.policy["sources"][0]["path"]).read_bytes(), b"competing output")


if __name__ == "__main__":
    unittest.main()
