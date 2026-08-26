from __future__ import annotations

import json
import tempfile
import unittest
import wave
from pathlib import Path

from scripts.validate_product_stem_handoff import (
    ARTIFACT_CONTRACT,
    BOUNDARY,
    EXPECTED_LIMITATION,
    MATERIAL_STATUS,
    RECONSTRUCTION_RULE,
    RECONSTRUCTION_SCHEMA,
    SCHEMA,
    SCHEMA_VERSION,
    sha256_file,
    validate_published_proof,
)


class ProductStemHandoffValidationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.samples = {
            "stem_drums": [110, -110, 220, -220],
            "stem_music": [330, -330, 440, -440],
            "stem_bass": [-40, 40, -60, 60],
        }
        self.samples["full_grid_mix"] = [
            sum(values)
            for values in zip(
                self.samples["stem_drums"],
                self.samples["stem_music"],
                self.samples["stem_bass"],
                strict=True,
            )
        ]
        self.proof = self._write_valid_bundle()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_valid_hash_identical_pcm_sum_bundle_passes(self) -> None:
        validate_published_proof(self.proof)

    def test_hash_drift_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["artifacts"][0]["sha256"] = "0" * 64
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            validate_published_proof(self.proof)

    def test_artifact_symlink_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        artifact_path = self.root / proof["artifacts"][0]["path"]
        target_path = self.root / "symlink-target.wav"
        target_path.write_bytes(artifact_path.read_bytes())
        artifact_path.unlink()
        artifact_path.symlink_to(target_path)

        with self.assertRaisesRegex(ValueError, "regular contained file"):
            validate_published_proof(self.proof)

    def test_path_traversal_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["artifacts"][0]["path"] = "../outside.wav"
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "media/path mismatch"):
            validate_published_proof(self.proof)

    def test_hash_valid_but_non_reconstructing_stem_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        artifact = proof["artifacts"][0]
        artifact_path = self.root / artifact["path"]
        self._write_wave(artifact_path, [15_000, -15_000, 15_000, -15_000])
        artifact["sha256"] = sha256_file(artifact_path)
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "reconstruction tolerance"):
            validate_published_proof(self.proof)

    def test_primitive_lane_cannot_be_silently_promoted(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["primitive_renderer_boundary"][
            "product_output_allowed"
        ] = True
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "product_output_allowed"):
            validate_published_proof(self.proof)

    def test_reconstruction_tolerance_cannot_be_relaxed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["reconstruction"]["max_allowed_abs_error"] = 1.0
        proof["reconstruction"]["max_allowed_rms_error"] = 1.0
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "not the frozen tolerance"):
            validate_published_proof(self.proof)

    def test_declared_reconstruction_metrics_must_match_audio(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["reconstruction"]["max_abs_error"] = 0.00001
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "declared maximum reconstruction error"):
            validate_published_proof(self.proof)

    def _write_valid_bundle(self) -> Path:
        artifacts = []
        for role, source_role, relative_path, origin in ARTIFACT_CONTRACT:
            path = self.root / relative_path
            self._write_wave(path, self.samples[role])
            artifacts.append(
                {
                    "role": role,
                    "source_role": source_role,
                    "path": relative_path,
                    "media_type": "audio/wav",
                    "sha256": sha256_file(path),
                    "origin": origin,
                }
            )
        proof = {
            "schema": SCHEMA,
            "schema_version": SCHEMA_VERSION,
            "boundary": BOUNDARY,
            "pack_id": "feral-grid-demo",
            "material_status": MATERIAL_STATUS,
            "release_ready": False,
            "musician_export_action_ready": False,
            "source_sha256": "a" * 64,
            "normalized_manifest_sha256": "b" * 64,
            "grid": {
                "sample_rate_hz": 48_000,
                "channel_count": 2,
                "bpm": 11_520_000.0,
                "beats_per_bar": 4,
                "bars": 2,
                "total_beats": 8,
                "frame_count": 2,
                "duration_seconds": 2 / 48_000,
            },
            "artifacts": artifacts,
            "reconstruction": {
                "schema": RECONSTRUCTION_SCHEMA,
                "rule": RECONSTRUCTION_RULE,
                "passed": True,
                "sample_rate_hz": 48_000,
                "channel_count": 2,
                "frame_count": 2,
                "max_abs_error": 0.0,
                "rms_error": 0.0,
                "max_allowed_abs_error": 3.0 / 32_768.0,
                "max_allowed_rms_error": 1.5 / 32_768.0,
            },
            "renderer_status": {
                "primitive_renderer_boundary": {
                    "schema": "riotbox.primitive_renderer_boundary.v1",
                    "evidence_role": "non_product_diagnostic_control",
                    "product_output_allowed": False,
                    "quality_proof": False,
                    "demo_readiness": "unverified",
                    "promotion_blocked": True,
                    "affected_paths": ["metrics.mc202_bass_pressure.pattern_origin"],
                    "musician_message": "fixture",
                },
                "limitations": [EXPECTED_LIMITATION],
            },
        }
        proof_path = self.root / "product_stem_handoff_proof.json"
        proof_path.write_text(json.dumps(proof))
        return proof_path

    @staticmethod
    def _write_wave(path: Path, samples: list[int]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with wave.open(str(path), "wb") as handle:
            handle.setnchannels(2)
            handle.setsampwidth(2)
            handle.setframerate(48_000)
            payload = b"".join(int(sample).to_bytes(2, "little", signed=True) for sample in samples)
            handle.writeframes(payload)


if __name__ == "__main__":
    unittest.main()
