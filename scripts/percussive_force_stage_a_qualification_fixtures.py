#!/usr/bin/env python3
"""Source-blind fixtures for the Stage-A qualification owner.

These fixtures build PCM WAV payloads in memory. They do not open, enumerate,
hash, render, or play any development, holdout, or reference audio path.
"""

from __future__ import annotations

import hashlib
import struct
from pathlib import Path
from unittest.mock import patch

import numpy as np

import run_percussive_force_stage_a_qualification as qualification
from source_holdout_development_access import SourceIdentity


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def pcm_wave(
    integers: tuple[int, ...],
    *,
    sample_rate_hz: int,
    sample_width_bits: int,
) -> bytes:
    if sample_width_bits == 16:
        sample_bytes = b"".join(struct.pack("<h", value) for value in integers)
    elif sample_width_bits == 24:
        sample_bytes = b"".join(
            int(value & 0xFF_FFFF).to_bytes(3, "little", signed=False)
            for value in integers
        )
    else:
        raise AssertionError("fixture supports only PCM16 and PCM24")
    channels = 1
    block_align = channels * (sample_width_bits // 8)
    fmt = struct.pack(
        "<HHIIHH",
        1,
        channels,
        sample_rate_hz,
        sample_rate_hz * block_align,
        block_align,
        sample_width_bits,
    )
    data_chunk = b"data" + struct.pack("<I", len(sample_bytes)) + sample_bytes
    if len(sample_bytes) & 1:
        data_chunk += b"\0"
    wave_payload = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt + data_chunk
    return b"RIFF" + struct.pack("<I", len(wave_payload)) + wave_payload


def decode_fixture(
    integers: tuple[int, ...],
    *,
    sample_rate_hz: int,
    sample_width_bits: int,
    expected_f32_sha256: str,
) -> None:
    payload = pcm_wave(
        integers,
        sample_rate_hz=sample_rate_hz,
        sample_width_bits=sample_width_bits,
    )
    raw_sha256 = hashlib.sha256(payload).hexdigest()
    source_format = {
        "sample_rate_hz": sample_rate_hz,
        "channels": 1,
        "sample_width_bits": sample_width_bits,
        "compression_type": "NONE",
        "maximum_duration_seconds": 16,
    }
    spec = qualification.FrozenSourceSpec(
        case_id=f"synthetic_pcm{sample_width_bits}",
        source_family="synthetic_source_blind",
        author="fixture",
        source_path=f"memory/synthetic_pcm{sample_width_bits}.wav",
        source_sha256=raw_sha256,
        license="synthetic-test-only",
        partition="development",
        source_format=source_format,
    )
    identity = SourceIdentity(
        case_id=spec.case_id,
        source_path=spec.source_path,
        expected_sha256=raw_sha256,
        partition="development",
        source_format=source_format,
    )
    access_record = {
        "case_id": spec.case_id,
        "source_path": spec.source_path,
        "expected_sha256": raw_sha256,
        "actual_sha256": raw_sha256,
        "access_verification_status": "verified_and_delivered_to_owner",
    }
    source, binding = qualification.decode_captured_source(
        qualification.CapturedSource(identity, payload, access_record),
        spec,
    )
    divisor = float(1 << (sample_width_bits - 1))
    expected = np.asarray(integers, dtype=np.float64) / divisor
    require(
        np.array_equal(source.samples[:, 0], expected),
        f"PCM{sample_width_bits} exact integer normalization drifted",
    )
    require(
        binding["pcm_f32le_sha256"] == expected_f32_sha256,
        f"PCM{sample_width_bits} Python/Rust f32-domain hash drifted",
    )
    require(
        source.input_lsb == 1.0 / divisor,
        f"PCM{sample_width_bits} input LSB drifted",
    )


def capture_owner_fixture() -> None:
    payload = pcm_wave((0, 1, -1), sample_rate_hz=48_000, sample_width_bits=16)
    raw_sha256 = hashlib.sha256(payload).hexdigest()
    source_format = {
        "sample_rate_hz": 48_000,
        "channels": 1,
        "sample_width_bits": 16,
        "compression_type": "NONE",
        "maximum_duration_seconds": 16,
    }
    spec = qualification.FrozenSourceSpec(
        case_id="synthetic_owner",
        source_family="synthetic_source_blind",
        author="fixture",
        source_path="memory/synthetic_owner.wav",
        source_sha256=raw_sha256,
        license="synthetic-test-only",
        partition="development",
        source_format=source_format,
    )
    identity = SourceIdentity(
        case_id=spec.case_id,
        source_path=spec.source_path,
        expected_sha256=raw_sha256,
        partition="development",
        source_format=source_format,
    )
    owner = qualification.SourceCaptureOwner((spec,))
    owner(
        identity,
        payload,
        {
            "actual_sha256": raw_sha256,
            "access_verification_status": "verified",
        },
    )
    require(len(owner.captured) == 1, "owner did not capture exactly one payload")
    try:
        owner(identity, payload, {"actual_sha256": raw_sha256})
    except qualification.QualificationSessionError:
        pass
    else:
        raise AssertionError("owner accepted an extra delivery")


def implementation_snapshot_fixture() -> None:
    first = qualification.implementation_snapshot()
    second = qualification.implementation_snapshot()
    require(first == second, "implementation snapshot is not deterministic")
    require(
        first["schema"]
        == "riotbox.percussive_force_stage_a_implementation_snapshot.v1",
        "implementation snapshot schema drifted",
    )
    require(
        len(first["files"]) == len(qualification.IMPLEMENTATION_PATHS),
        "implementation snapshot omitted files",
    )


def closed_v1_execution_fixture() -> None:
    reached: list[str] = []

    def forbidden(name: str) -> object:
        def callback(*_args: object, **_kwargs: object) -> None:
            reached.append(name)
            raise AssertionError(f"closed v1 runner reached {name}")

        return callback

    with (
        patch.object(
            qualification,
            "validate_session_directory",
            forbidden("session_validation"),
        ),
        patch.object(
            qualification,
            "validate_repository",
            forbidden("contract_validation"),
        ),
        patch.object(
            qualification,
            "run_source_blind_preflight",
            forbidden("subprocess_preflight"),
        ),
        patch.object(
            qualification,
            "verify_development_source_files",
            forbidden("safe_access_gate"),
        ),
    ):
        try:
            qualification.run_session(Path("/must-not-be-opened-stage-a-v1"))
        except qualification.StageAV1ExecutionClosed as error:
            require(
                error.code == qualification.STAGE_A_V1_EXECUTION_CLOSED_CODE,
                "closed v1 runner returned the wrong typed refusal",
            )
        else:
            raise AssertionError("closed v1 runner accepted a new session")
    require(not reached, f"closed v1 runner reached callbacks: {reached}")


def main() -> int:
    decode_fixture(
        (-32_768, -1, 0, 1, 32_767),
        sample_rate_hz=48_000,
        sample_width_bits=16,
        expected_f32_sha256=(
            "be2abff76fe003c0c2471d736606215695ea777e75049ec1e920aa0d0ae57f2d"
        ),
    )
    decode_fixture(
        (-8_388_608, -1, 0, 1, 8_388_607),
        sample_rate_hz=44_100,
        sample_width_bits=24,
        expected_f32_sha256=(
            "14240b34f81c3fb475f52f9afa2473705c81259e2b2461022b16e5e654868537"
        ),
    )
    capture_owner_fixture()
    implementation_snapshot_fixture()
    closed_v1_execution_fixture()
    print("PASS: Stage-A qualification owner synthetic fixtures")
    print("source_audio_accessed=false")
    print("holdout_audio_accessed=false")
    print("commercial_reference_accessed=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
