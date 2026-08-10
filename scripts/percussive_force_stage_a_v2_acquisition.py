#!/usr/bin/env python3
"""Source-blind primitives for the RIOTBOX-1430 Stage-A-v2 acquisition gate.

This module deliberately owns no provider URLs and performs no network access by
itself.  It can stream an already authorized response body to one caller-owned
file descriptor and inspect only RIFF/WAVE container headers.  It never decodes,
iterates, measures, renders, or plays PCM samples.
"""

from __future__ import annotations

import ctypes
import errno
import hashlib
import ipaddress
import os
import re
import struct
from typing import Any, BinaryIO
from urllib.parse import unquote, urlsplit


FORMAT_CONTRACT = {
    "container": "RIFF/WAVE",
    "format_tag": 1,
    "pcm_encoding": "signed_little_endian_integer",
    "sample_rate_hz_inclusive": [32_000, 192_000],
    "channel_counts": [1, 2],
    "sample_width_bits": [16, 24],
    "maximum_duration_seconds": 16,
    "maximum_container_overhead_bytes": 1_048_576,
    "wave_format_extensible_allowed": False,
    "sample_decode_or_iteration_allowed": False,
}
MAX_RIFF_CHUNKS = 256
STREAM_CHUNK_BYTES = 65_536
PROVIDER_HOST = "opengameart.org"
ALLOWED_WAV_MEDIA_TYPES = {
    "application/octet-stream",
    "application/x-wav",
    "audio/vnd.wave",
    "audio/wav",
    "audio/wave",
    "audio/x-wav",
}
DECIMAL = re.compile(r"^[0-9]+$")
SAFE_PATH_COMPONENT = re.compile(r"^[a-zA-Z0-9][a-zA-Z0-9._-]*$")
RENAME_NOREPLACE = 1


class AcquisitionError(ValueError):
    """Raised when the bounded acquisition or header contract fails closed."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AcquisitionError(message)


def validate_format_contract(value: Any, prefix: str = "format_contract") -> None:
    require(isinstance(value, dict), f"{prefix} must be an object")
    require(value.keys() == FORMAT_CONTRACT.keys(), f"{prefix} key set changed")
    for key, expected in FORMAT_CONTRACT.items():
        actual = value.get(key)
        require(
            type(actual) is type(expected) and actual == expected,
            f"{prefix}.{key} must be {expected!r}",
        )


def maximum_declared_attachment_bytes() -> int:
    rate = FORMAT_CONTRACT["sample_rate_hz_inclusive"][1]
    channels = max(FORMAT_CONTRACT["channel_counts"])
    sample_bytes = max(FORMAT_CONTRACT["sample_width_bits"]) // 8
    duration = FORMAT_CONTRACT["maximum_duration_seconds"]
    overhead = FORMAT_CONTRACT["maximum_container_overhead_bytes"]
    return rate * channels * sample_bytes * duration + overhead


def validate_declared_attachment_byte_count(value: Any, prefix: str) -> int:
    require(
        isinstance(value, int) and not isinstance(value, bool) and value >= 44,
        f"{prefix} must be an integer large enough for PCM RIFF/WAVE",
    )
    require(
        value <= maximum_declared_attachment_bytes(),
        f"{prefix} exceeds the frozen PCM-duration and RIFF-overhead envelope",
    )
    return value


def validate_provider_url(value: Any, *, kind: str, prefix: str) -> str:
    require(isinstance(value, str) and value, f"{prefix} must be a non-empty URL")
    require(value.strip() == value, f"{prefix} must not contain outer whitespace")
    require("\\" not in value and "\x00" not in value, f"{prefix} contains a forbidden byte")
    parsed = urlsplit(value)
    require(parsed.scheme == "https", f"{prefix} must use HTTPS")
    require(parsed.hostname == PROVIDER_HOST, f"{prefix} must use the pinned provider host")
    require(parsed.port is None, f"{prefix} must not declare a non-default port")
    require(parsed.username is None and parsed.password is None, f"{prefix} must not contain userinfo")
    require(not parsed.query and not parsed.fragment, f"{prefix} must not contain query or fragment")
    require("//" not in parsed.path, f"{prefix} path must not contain empty components")
    decoded_path = unquote(parsed.path)
    require(
        "\\" not in decoded_path and "\x00" not in decoded_path,
        f"{prefix} decoded path contains a forbidden byte",
    )
    require(
        all(component not in {"", ".", ".."} for component in decoded_path.split("/")[1:]),
        f"{prefix} path must not contain empty, dot, or parent components",
    )
    if kind == "page":
        require(parsed.path.startswith("/content/"), f"{prefix} must be one exact content page")
    elif kind == "download":
        require(
            parsed.path.startswith("/sites/default/files/")
            and decoded_path.casefold().endswith(".wav"),
            f"{prefix} must be one direct provider WAV attachment",
        )
    elif kind == "profile":
        require(parsed.path.startswith("/users/"), f"{prefix} must be one exact author profile")
    else:
        raise AcquisitionError(f"{prefix}: unsupported provider URL kind {kind!r}")
    return value


def validate_public_peer_ip(value: Any, prefix: str = "peer_ip") -> str:
    require(isinstance(value, str) and value, f"{prefix} must be a non-empty IP address")
    try:
        address = ipaddress.ip_address(value)
    except ValueError as error:
        raise AcquisitionError(f"{prefix} is not an IP address") from error
    require(
        address.is_global and not address.is_multicast,
        f"{prefix} must be globally routable unicast",
    )
    if isinstance(address, ipaddress.IPv6Address):
        require(
            not address.is_site_local,
            f"{prefix} must not use deprecated IPv6 site-local space",
        )
    if isinstance(address, ipaddress.IPv6Address) and address.ipv4_mapped is not None:
        require(
            address.ipv4_mapped.is_global
            and not address.ipv4_mapped.is_multicast,
            f"{prefix} mapped IPv4 must be globally routable unicast",
        )
    return address.compressed


def validate_response_gate(
    *,
    status: Any,
    headers: dict[str, list[str]],
    expected_byte_count: int,
) -> dict[str, Any]:
    """Validate already received HTTP metadata before any body byte is read."""

    validate_declared_attachment_byte_count(
        expected_byte_count, "expected_attachment_byte_count"
    )
    require(type(status) is int and status == 200, "HTTP response status must be exactly 200")
    normalized: dict[str, list[str]] = {}
    for name, values in headers.items():
        require(isinstance(name, str), "HTTP header names must be strings")
        require(
            isinstance(values, list) and all(isinstance(item, str) for item in values),
            f"HTTP header {name!r} must preserve its raw value list",
        )
        lowered = name.casefold()
        require(lowered not in normalized, f"HTTP header {name!r} was normalized twice")
        normalized[lowered] = values

    lengths = normalized.get("content-length", [])
    require(len(lengths) == 1, "HTTP response must contain exactly one Content-Length")
    raw_length = lengths[0]
    require(
        DECIMAL.fullmatch(raw_length) is not None,
        "HTTP Content-Length must be one unsigned decimal integer",
    )
    require(
        int(raw_length, 10) == expected_byte_count,
        "HTTP Content-Length differs from the preregistered attachment byte count",
    )
    require(
        not normalized.get("transfer-encoding"),
        "HTTP Transfer-Encoding is forbidden",
    )
    encodings = normalized.get("content-encoding", [])
    require(
        not encodings or encodings == ["identity"],
        "HTTP Content-Encoding must be absent or exactly identity",
    )
    media_types = normalized.get("content-type", [])
    require(len(media_types) == 1, "HTTP response must contain exactly one Content-Type")
    media_type = media_types[0].split(";", 1)[0].strip().casefold()
    require(media_type in ALLOWED_WAV_MEDIA_TYPES, "HTTP Content-Type is not an allowed WAV type")
    return {
        "response_status": status,
        "content_length_raw_values": lengths,
        "content_encoding_raw_values": encodings,
        "transfer_encoding_raw_values": normalized.get("transfer-encoding", []),
        "content_type_raw_values": media_types,
        "normalized_media_type": media_type,
    }


def rename_directory_noreplace(
    source_parent_fd: int,
    source_name: str,
    destination_parent_fd: int,
    destination_name: str,
) -> None:
    """Atomically publish one complete batch directory without replacement."""

    for label, value in (
        ("source_name", source_name),
        ("destination_name", destination_name),
    ):
        require(
            isinstance(value, str)
            and SAFE_PATH_COMPONENT.fullmatch(value) is not None
            and value not in {".", ".."},
            f"{label} must be one safe path component",
        )
    libc = ctypes.CDLL(None, use_errno=True)
    renameat2 = getattr(libc, "renameat2", None)
    require(renameat2 is not None, "Linux renameat2 is required for atomic publication")
    renameat2.argtypes = [
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    ]
    renameat2.restype = ctypes.c_int
    result = renameat2(
        source_parent_fd,
        os.fsencode(source_name),
        destination_parent_fd,
        os.fsencode(destination_name),
        RENAME_NOREPLACE,
    )
    if result != 0:
        error_number = ctypes.get_errno()
        if error_number == errno.ENOSYS:
            raise AcquisitionError(
                "kernel renameat2 support is required for atomic publication"
            )
        raise OSError(
            error_number,
            os.strerror(error_number),
            f"{source_name} -> {destination_name}",
        )


def stream_exact_attachment(
    body: BinaryIO,
    destination_fd: int,
    expected_byte_count: int,
) -> dict[str, Any]:
    """Stream and hash exactly one declared attachment, reading at most +1 byte.

    ``body`` is intentionally a narrow binary reader rather than a URL.  The
    future network owner must validate HTTP status and headers before passing a
    response here.  Bytes are copied and hashed only; they are never interpreted
    as samples.
    """

    validate_declared_attachment_byte_count(
        expected_byte_count, "expected_attachment_byte_count"
    )
    digest = hashlib.sha256()
    consumed = 0
    while consumed < expected_byte_count:
        remaining = expected_byte_count - consumed
        chunk = body.read(min(STREAM_CHUNK_BYTES, remaining))
        require(isinstance(chunk, bytes), "response body reader must return bytes")
        require(chunk, "response body ended before the declared attachment byte count")
        require(
            len(chunk) <= remaining,
            "response body reader exceeded the requested bounded read",
        )
        _write_all(destination_fd, chunk)
        digest.update(chunk)
        consumed += len(chunk)

    extra = body.read(1)
    require(isinstance(extra, bytes), "response body reader must return bytes")
    require(
        extra == b"",
        "response body contains at least one byte beyond the declared attachment",
    )
    os.fsync(destination_fd)
    return {
        "body_read_cap": expected_byte_count + 1,
        "body_bytes_consumed": consumed,
        "actual_sha256": digest.hexdigest(),
        "sample_decode_performed": False,
        "pcm_sample_iteration_performed": False,
    }


def _write_all(file_descriptor: int, payload: bytes) -> None:
    offset = 0
    while offset < len(payload):
        written = os.write(file_descriptor, payload[offset:])
        require(written > 0, "quarantine write made no progress")
        offset += written


def inspect_riff_pcm_header_only(
    file_descriptor: int,
    expected_byte_count: int,
) -> dict[str, Any]:
    """Validate strict native PCM by reading container headers, never samples."""

    validate_declared_attachment_byte_count(
        expected_byte_count, "expected_attachment_byte_count"
    )
    metadata = os.fstat(file_descriptor)
    require(
        metadata.st_size == expected_byte_count,
        "quarantine size differs from the declared attachment byte count",
    )
    header = _pread_exact(file_descriptor, 0, 12, "RIFF/WAVE header")
    require(header[0:4] == b"RIFF", "container must be little-endian RIFF")
    require(header[8:12] == b"WAVE", "RIFF form type must be WAVE")
    riff_size = struct.unpack_from("<I", header, 4)[0]
    require(
        riff_size + 8 == expected_byte_count,
        "RIFF size must cover the exact declared attachment without trailing bytes",
    )

    offset = 12
    chunk_count = 0
    chunks: list[dict[str, Any]] = []
    fmt_record: dict[str, Any] | None = None
    data_record: dict[str, Any] | None = None
    while offset < expected_byte_count:
        chunk_count += 1
        require(chunk_count <= MAX_RIFF_CHUNKS, "RIFF chunk count exceeds the frozen bound")
        require(
            offset <= expected_byte_count - 8,
            "RIFF ends with a truncated chunk header",
        )
        chunk_header = _pread_exact(
            file_descriptor,
            offset,
            8,
            f"RIFF chunk header {chunk_count}",
        )
        chunk_id = chunk_header[0:4]
        require(
            all(32 <= byte <= 126 for byte in chunk_id),
            "RIFF chunk identifiers must be printable ASCII",
        )
        chunk_size = struct.unpack_from("<I", chunk_header, 4)[0]
        payload_offset = offset + 8
        payload_end = payload_offset + chunk_size
        padded_end = payload_end + (chunk_size & 1)
        require(
            payload_end >= payload_offset and padded_end >= payload_end,
            "RIFF chunk offset arithmetic overflowed",
        )
        require(
            padded_end <= expected_byte_count,
            "RIFF chunk payload or odd-byte padding exceeds the container",
        )
        chunk_name = chunk_id.decode("ascii")
        chunks.append(
            {
                "chunk_id": chunk_name,
                "header_offset": offset,
                "payload_offset": payload_offset,
                "payload_size": chunk_size,
                "padding_bytes": chunk_size & 1,
                "payload_read_for_header_validation": chunk_id == b"fmt ",
            }
        )

        if chunk_id == b"fmt ":
            require(fmt_record is None, "RIFF/WAVE must contain exactly one fmt chunk")
            require(data_record is None, "RIFF/WAVE fmt chunk must precede the data chunk")
            require(chunk_size == 16, "PCM fmt chunk must be exactly 16 bytes")
            fmt_payload = _pread_exact(
                file_descriptor,
                payload_offset,
                16,
                "PCM fmt payload",
            )
            (
                format_tag,
                channels,
                sample_rate_hz,
                byte_rate,
                block_align,
                sample_width_bits,
            ) = struct.unpack("<HHIIHH", fmt_payload)
            _validate_pcm_format(
                format_tag=format_tag,
                channels=channels,
                sample_rate_hz=sample_rate_hz,
                byte_rate=byte_rate,
                block_align=block_align,
                sample_width_bits=sample_width_bits,
            )
            fmt_record = {
                "fmt_offset": payload_offset,
                "fmt_size": chunk_size,
                "format_tag": format_tag,
                "pcm_encoding": "signed_little_endian_integer",
                "channels": channels,
                "sample_rate_hz": sample_rate_hz,
                "byte_rate": byte_rate,
                "block_align": block_align,
                "sample_width_bits": sample_width_bits,
                "valid_bits": sample_width_bits,
            }
        elif chunk_id == b"data":
            require(fmt_record is not None, "RIFF/WAVE data chunk must follow fmt")
            require(data_record is None, "RIFF/WAVE must contain exactly one data chunk")
            require(chunk_size > 0, "PCM data chunk must contain at least one frame")
            block_align = fmt_record["block_align"]
            require(
                chunk_size % block_align == 0,
                "PCM data size must contain only complete frames",
            )
            frame_count = chunk_size // block_align
            maximum_frames = (
                fmt_record["sample_rate_hz"]
                * FORMAT_CONTRACT["maximum_duration_seconds"]
            )
            require(
                frame_count <= maximum_frames,
                "PCM duration exceeds the frozen 16-second acquisition maximum",
            )
            data_record = {
                "data_offset": payload_offset,
                "data_size": chunk_size,
                "frame_count": frame_count,
                "duration_numerator_frames": frame_count,
                "duration_denominator_sample_rate": fmt_record["sample_rate_hz"],
            }

        require(padded_end > offset, "RIFF chunk traversal made no progress")
        offset = padded_end

    require(offset == expected_byte_count, "RIFF chunks do not cover the exact container")
    require(fmt_record is not None, "RIFF/WAVE is missing its fmt chunk")
    require(data_record is not None, "RIFF/WAVE is missing its data chunk")
    container_overhead = expected_byte_count - data_record["data_size"]
    require(
        container_overhead <= FORMAT_CONTRACT["maximum_container_overhead_bytes"],
        "RIFF non-audio container overhead exceeds the frozen bound",
    )
    return {
        "container": "RIFF/WAVE",
        "riff_size": riff_size,
        **fmt_record,
        **data_record,
        "container_overhead_bytes": container_overhead,
        "chunk_table": chunks,
        "header_validation_scope": "container_headers_only_no_sample_payload_reads",
        "sample_decode_performed": False,
        "pcm_sample_iteration_performed": False,
    }


def _validate_pcm_format(
    *,
    format_tag: int,
    channels: int,
    sample_rate_hz: int,
    byte_rate: int,
    block_align: int,
    sample_width_bits: int,
) -> None:
    require(format_tag == FORMAT_CONTRACT["format_tag"], "WAVE format_tag must be PCM 1")
    require(
        channels in FORMAT_CONTRACT["channel_counts"],
        "PCM channel count is outside the frozen supported set",
    )
    minimum_rate, maximum_rate = FORMAT_CONTRACT["sample_rate_hz_inclusive"]
    require(
        minimum_rate <= sample_rate_hz <= maximum_rate,
        "PCM sample rate is outside the frozen inclusive range",
    )
    require(
        sample_width_bits in FORMAT_CONTRACT["sample_width_bits"],
        "PCM sample width must be exactly 16 or 24 bits",
    )
    expected_block_align = channels * (sample_width_bits // 8)
    require(block_align == expected_block_align, "PCM block_align is inconsistent")
    require(
        byte_rate == sample_rate_hz * block_align,
        "PCM byte_rate is inconsistent",
    )


def _pread_exact(
    file_descriptor: int,
    offset: int,
    byte_count: int,
    label: str,
) -> bytes:
    payload = os.pread(file_descriptor, byte_count, offset)
    require(len(payload) == byte_count, f"{label} is truncated")
    return payload
