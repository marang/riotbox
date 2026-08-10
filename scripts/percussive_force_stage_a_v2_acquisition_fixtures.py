#!/usr/bin/env python3
"""Source-blind fixtures for the Stage-A-v2 acquisition primitives."""

from __future__ import annotations

import hashlib
import io
import os
import struct
import tempfile
from pathlib import Path
from typing import Callable

import percussive_force_stage_a_v2_acquisition as acquisition


def riff_pcm(
    *,
    sample_rate_hz: int = 48_000,
    channels: int = 2,
    sample_width_bits: int = 16,
    frame_count: int = 64,
    format_tag: int = 1,
    fmt_size: int = 16,
    fmt_before_data: bool = True,
    duplicate_fmt: bool = False,
    duplicate_data: bool = False,
    byte_rate_delta: int = 0,
    block_align_delta: int = 0,
    data_size_delta: int = 0,
    metadata_payload: bytes = b"",
) -> bytes:
    block_align = channels * (sample_width_bits // 8)
    byte_rate = sample_rate_hz * block_align
    fmt_payload = struct.pack(
        "<HHIIHH",
        format_tag,
        channels,
        sample_rate_hz,
        byte_rate + byte_rate_delta,
        block_align + block_align_delta,
        sample_width_bits,
    )
    if fmt_size > 16:
        fmt_payload += b"\x00" * (fmt_size - 16)
    fmt_chunk = chunk(b"fmt ", fmt_payload)
    data_payload = b"\x00" * max(0, frame_count * block_align + data_size_delta)
    data_chunk = chunk(b"data", data_payload)
    chunks: list[bytes] = []
    if metadata_payload:
        chunks.append(chunk(b"LIST", metadata_payload))
    chunks.extend([fmt_chunk, data_chunk] if fmt_before_data else [data_chunk, fmt_chunk])
    if duplicate_fmt:
        chunks.append(fmt_chunk)
    if duplicate_data:
        chunks.append(data_chunk)
    body = b"WAVE" + b"".join(chunks)
    return b"RIFF" + struct.pack("<I", len(body)) + body


def chunk(chunk_id: bytes, payload: bytes) -> bytes:
    padding = b"\x00" if len(payload) & 1 else b""
    return chunk_id + struct.pack("<I", len(payload)) + payload + padding


def expect_fail(name: str, operation: Callable[[], None], token: str) -> None:
    try:
        operation()
    except acquisition.AcquisitionError as error:
        if token not in str(error):
            raise AssertionError(
                f"{name}: wrong failure {error!s}; expected {token!r}"
            ) from error
        return
    raise AssertionError(f"{name}: unexpectedly passed")


def inspect_payload(payload: bytes) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-v2-riff-") as temporary:
        path = Path(temporary) / "synthetic.wav"
        path.write_bytes(payload)
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
        try:
            return acquisition.inspect_riff_pcm_header_only(descriptor, len(payload))
        finally:
            os.close(descriptor)


def run_header_fixtures() -> int:
    valid = riff_pcm(metadata_payload=b"abc")
    record = inspect_payload(valid)
    if (
        record["format_tag"] != 1
        or record["sample_rate_hz"] != 48_000
        or record["channels"] != 2
        or record["sample_width_bits"] != 16
        or record["frame_count"] != 64
        or record["sample_decode_performed"] is not False
    ):
        raise AssertionError("valid PCM16 header returned the wrong frozen facts")

    valid_24 = riff_pcm(sample_rate_hz=44_100, channels=1, sample_width_bits=24)
    if inspect_payload(valid_24)["valid_bits"] != 24:
        raise AssertionError("valid PCM24 header was not admitted")

    with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-v2-no-samples-") as temporary:
        path = Path(temporary) / "synthetic.wav"
        path.write_bytes(valid)
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
        real_pread = acquisition.os.pread
        reads: list[tuple[int, int]] = []

        def observed_pread(fd: int, byte_count: int, offset: int) -> bytes:
            reads.append((offset, byte_count))
            return real_pread(fd, byte_count, offset)

        acquisition.os.pread = observed_pread
        try:
            observed = acquisition.inspect_riff_pcm_header_only(descriptor, len(valid))
        finally:
            acquisition.os.pread = real_pread
            os.close(descriptor)
        data_start = int(observed["data_offset"])
        data_end = data_start + int(observed["data_size"])
        if any(
            max(offset, data_start) < min(offset + byte_count, data_end)
            for offset, byte_count in reads
        ):
            raise AssertionError("header-only parser read PCM sample payload bytes")

    mutations: list[tuple[str, bytes, str]] = [
        ("rifx", b"RIFX" + valid[4:], "little-endian RIFF"),
        ("rf64", b"RF64" + valid[4:], "little-endian RIFF"),
        ("not_wave", valid[:8] + b"AVI " + valid[12:], "form type"),
        ("trailing_byte", valid + b"x", "RIFF size"),
        ("fmt_18", riff_pcm(fmt_size=18), "exactly 16 bytes"),
        ("float", riff_pcm(format_tag=3), "format_tag"),
        ("extensible", riff_pcm(format_tag=0xFFFE), "format_tag"),
        ("data_before_fmt", riff_pcm(fmt_before_data=False), "must follow fmt"),
        ("duplicate_fmt", riff_pcm(duplicate_fmt=True), "exactly one fmt"),
        ("duplicate_data", riff_pcm(duplicate_data=True), "exactly one data"),
        ("three_channels", riff_pcm(channels=3), "channel count"),
        ("low_rate", riff_pcm(sample_rate_hz=31_999), "sample rate"),
        ("high_rate", riff_pcm(sample_rate_hz=192_001), "sample rate"),
        ("eight_bit", riff_pcm(sample_width_bits=8), "sample width"),
        ("wrong_byte_rate", riff_pcm(byte_rate_delta=1), "byte_rate"),
        ("wrong_block_align", riff_pcm(block_align_delta=1), "block_align"),
        ("partial_frame", riff_pcm(data_size_delta=-1), "complete frames"),
        ("empty_data", riff_pcm(frame_count=0), "at least one frame"),
        (
            "over_duration",
            riff_pcm(sample_rate_hz=32_000, channels=1, frame_count=32_000 * 16 + 1),
            "duration exceeds",
        ),
    ]
    for name, payload, token in mutations:
        expect_fail(name, lambda payload=payload: inspect_payload(payload), token)

    truncated_chunk = valid[:-1]
    truncated_chunk = (
        truncated_chunk[:4]
        + struct.pack("<I", len(truncated_chunk) - 8)
        + truncated_chunk[8:]
    )
    expect_fail(
        "truncated_chunk",
        lambda: inspect_payload(truncated_chunk),
        "exceeds the container",
    )
    return len(mutations) + 1


def run_stream_fixtures() -> int:
    payload = riff_pcm(frame_count=256)
    expected_hash = hashlib.sha256(payload).hexdigest()
    with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-v2-stream-") as temporary:
        path = Path(temporary) / "quarantine.wav"
        descriptor = os.open(
            path,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW,
            0o600,
        )
        try:
            record = acquisition.stream_exact_attachment(
                io.BytesIO(payload), descriptor, len(payload)
            )
        finally:
            os.close(descriptor)
        if (
            record["actual_sha256"] != expected_hash
            or record["body_bytes_consumed"] != len(payload)
            or record["body_read_cap"] != len(payload) + 1
            or path.read_bytes() != payload
        ):
            raise AssertionError("exact streaming did not preserve byte identity")

    def stream_failure(body: bytes, expected: int) -> None:
        with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-v2-stream-fail-") as temporary:
            path = Path(temporary) / "quarantine.wav"
            descriptor = os.open(
                path,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW,
                0o600,
            )
            try:
                acquisition.stream_exact_attachment(io.BytesIO(body), descriptor, expected)
            finally:
                os.close(descriptor)

    expect_fail(
        "early_eof",
        lambda: stream_failure(payload[:-1], len(payload)),
        "ended before",
    )
    expect_fail(
        "one_extra_byte",
        lambda: stream_failure(payload + b"x", len(payload)),
        "beyond the declared attachment",
    )
    return 2


def run_network_metadata_fixtures() -> int:
    page = "https://opengameart.org/content/exact-page"
    profile = "https://opengameart.org/users/exact-author"
    download = "https://opengameart.org/sites/default/files/exact%20file.wav"
    acquisition.validate_provider_url(page, kind="page", prefix="page_url")
    acquisition.validate_provider_url(profile, kind="profile", prefix="profile_url")
    acquisition.validate_provider_url(download, kind="download", prefix="download_url")
    acquisition.validate_public_peer_ip("8.8.8.8")
    expected = 1_000
    good_headers = {
        "Content-Length": [str(expected)],
        "Content-Type": ["audio/x-wav"],
        "Content-Encoding": ["identity"],
    }
    record = acquisition.validate_response_gate(
        status=200,
        headers=good_headers,
        expected_byte_count=expected,
    )
    if record["normalized_media_type"] != "audio/x-wav":
        raise AssertionError("valid response metadata was not normalized")

    cases: list[tuple[str, Callable[[], None], str]] = [
        (
            "http_page",
            lambda: acquisition.validate_provider_url(
                page.replace("https:", "http:"), kind="page", prefix="page_url"
            ),
            "HTTPS",
        ),
        (
            "wrong_host",
            lambda: acquisition.validate_provider_url(
                download.replace("opengameart.org", "example.org"),
                kind="download",
                prefix="download_url",
            ),
            "provider host",
        ),
        (
            "redirect_status",
            lambda: acquisition.validate_response_gate(
                status=302, headers=good_headers, expected_byte_count=expected
            ),
            "exactly 200",
        ),
        (
            "partial_status",
            lambda: acquisition.validate_response_gate(
                status=206, headers=good_headers, expected_byte_count=expected
            ),
            "exactly 200",
        ),
        (
            "missing_length",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={"Content-Type": ["audio/wav"]},
                expected_byte_count=expected,
            ),
            "exactly one Content-Length",
        ),
        (
            "duplicate_length",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [str(expected), str(expected)],
                    "Content-Type": ["audio/wav"],
                },
                expected_byte_count=expected,
            ),
            "exactly one Content-Length",
        ),
        (
            "joined_length",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [f"{expected}, {expected}"],
                    "Content-Type": ["audio/wav"],
                },
                expected_byte_count=expected,
            ),
            "unsigned decimal",
        ),
        (
            "wrong_length",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [str(expected + 1)],
                    "Content-Type": ["audio/wav"],
                },
                expected_byte_count=expected,
            ),
            "differs",
        ),
        (
            "chunked",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [str(expected)],
                    "Content-Type": ["audio/wav"],
                    "Transfer-Encoding": ["chunked"],
                },
                expected_byte_count=expected,
            ),
            "Transfer-Encoding",
        ),
        (
            "gzip",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [str(expected)],
                    "Content-Type": ["audio/wav"],
                    "Content-Encoding": ["gzip"],
                },
                expected_byte_count=expected,
            ),
            "Content-Encoding",
        ),
        (
            "html_type",
            lambda: acquisition.validate_response_gate(
                status=200,
                headers={
                    "Content-Length": [str(expected)],
                    "Content-Type": ["text/html"],
                },
                expected_byte_count=expected,
            ),
            "allowed WAV type",
        ),
        (
            "private_peer",
            lambda: acquisition.validate_public_peer_ip("127.0.0.1"),
            "globally routable",
        ),
        (
            "ipv4_multicast_peer",
            lambda: acquisition.validate_public_peer_ip("224.0.0.1"),
            "globally routable unicast",
        ),
        (
            "ipv6_multicast_peer",
            lambda: acquisition.validate_public_peer_ip("ff02::1"),
            "globally routable unicast",
        ),
        (
            "ipv6_site_local_peer",
            lambda: acquisition.validate_public_peer_ip("fec0::1"),
            "site-local",
        ),
    ]
    for name, operation, token in cases:
        expect_fail(name, operation, token)
    return len(cases)


def run_atomic_publication_fixtures() -> int:
    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-v2-publication-") as temporary:
        root = Path(temporary)
        source_parent = root / "quarantine"
        destination_parent = root / "development"
        source_parent.mkdir()
        destination_parent.mkdir()
        batch = source_parent / "batch.incomplete"
        batch.mkdir()
        (batch / "one.wav").write_bytes(b"synthetic-one")
        source_fd = os.open(source_parent, directory_flags)
        destination_fd = os.open(destination_parent, directory_flags)
        try:
            acquisition.rename_directory_noreplace(
                source_fd,
                "batch.incomplete",
                destination_fd,
                "batch-v1",
            )
        finally:
            os.close(source_fd)
            os.close(destination_fd)
        if batch.exists() or (destination_parent / "batch-v1" / "one.wav").read_bytes() != b"synthetic-one":
            raise AssertionError("atomic directory publication lost the complete batch")

        second = source_parent / "second.incomplete"
        second.mkdir()
        (second / "two.wav").write_bytes(b"synthetic-two")
        source_fd = os.open(source_parent, directory_flags)
        destination_fd = os.open(destination_parent, directory_flags)
        try:
            try:
                acquisition.rename_directory_noreplace(
                    source_fd,
                    "second.incomplete",
                    destination_fd,
                    "batch-v1",
                )
            except FileExistsError:
                pass
            else:
                raise AssertionError("atomic publication replaced an existing batch")
        finally:
            os.close(source_fd)
            os.close(destination_fd)
        if not second.is_dir() or not (destination_parent / "batch-v1").is_dir():
            raise AssertionError("failed no-replace publication damaged either directory")
    return 1


def main() -> int:
    header_count = run_header_fixtures()
    stream_count = run_stream_fixtures()
    network_metadata_count = run_network_metadata_fixtures()
    publication_count = run_atomic_publication_fixtures()
    print(
        "PASS: source-blind acquisition primitives "
        f"({header_count} RIFF mutations, {stream_count} byte-stream mutations, "
        f"{network_metadata_count} network-metadata mutations, "
        f"{publication_count} atomic-publication fixture)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
