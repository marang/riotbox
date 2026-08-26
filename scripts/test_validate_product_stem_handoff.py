from __future__ import annotations

import json
import tempfile
import unittest
import wave
from pathlib import Path

from scripts.validate_product_stem_handoff import (
    ARTIFACT_CONTRACT,
    BOUNDARY,
    MATERIAL_STATUS,
    MC202_MIN_SOURCE_GRID_HIT_RATIO,
    MC202_SOURCE_EXPRESSION_SCHEMA,
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

    def test_primitive_lane_cannot_be_silently_reintroduced(self) -> None:
        proof = json.loads(self.proof.read_text())
        next(
            artifact
            for artifact in proof["artifacts"]
            if artifact["role"] == "stem_bass"
        )["origin"] = "primitive_renderer"
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "origin mismatch"):
            validate_published_proof(self.proof)

    def test_primitive_boundary_cannot_be_reintroduced(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["primitive_renderer_boundary"] = {
            "product_output_allowed": False
        }
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "stale or unknown"):
            validate_published_proof(self.proof)

    def test_unapplied_source_expression_plan_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["mc202_source_expression"][
            "source_expression_render_plan_applied"
        ] = False
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "render_plan_applied"):
            validate_published_proof(self.proof)

    def test_source_failure_fallback_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["mc202_source_expression"][
            "source_failure_fallback"
        ] = True
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "source_failure_fallback"):
            validate_published_proof(self.proof)

    def test_weak_source_contour_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["mc202_source_expression"][
            "source_contour_delta_rms"
        ] = 0.0
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "contour is below"):
            validate_published_proof(self.proof)

    def test_weak_source_grid_alignment_fails_closed(self) -> None:
        proof = json.loads(self.proof.read_text())
        proof["renderer_status"]["mc202_source_expression"][
            "source_grid_hit_ratio"
        ] = MC202_MIN_SOURCE_GRID_HIT_RATIO - 0.01
        self.proof.write_text(json.dumps(proof))

        with self.assertRaisesRegex(ValueError, "grid alignment"):
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
                "mc202_source_expression": {
                    "schema": MC202_SOURCE_EXPRESSION_SCHEMA,
                    "pattern_origin": "source_derived",
                    "bass_pressure_applied": True,
                    "bass_pressure_reason": "mc202_source_grid_proof_renderer",
                    "source_expression_render_plan_applied": True,
                    "source_expression_role": "bass_pressure",
                    "source_failure_fallback": False,
                    "source_contour_origin": "source_derived_contour",
                    "source_contour_applied": True,
                    "source_contour_delta_rms": 0.01,
                    "source_contour_min_required_delta_rms": 0.001,
                    "source_grid_hit_ratio": 0.75,
                    "source_grid_min_required_hit_ratio": MC202_MIN_SOURCE_GRID_HIT_RATIO,
                },
                "limitations": [],
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
