#!/usr/bin/env python3
"""Execute the one authorized RIOTBOX-1430 development-only acquisition.

This runner performs exactly three preregistered GET requests in order.  It
hashes raw response bytes and reads only RIFF/WAVE container headers.  It does
not decode or iterate PCM, compute source features/events, render, or play
audio, and it never opens holdout audio or commercial references.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import os
import socket
import ssl
import stat
import sys
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.parse import urlsplit

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_artifacts as artifacts
import percussive_force_stage_a_v2_acquisition_contract as batch_contract
import validate_percussive_force_stage_a_v2_acquisition_artifacts as artifact_validator
import validate_percussive_force_stage_a_v2_acquisition_batch as batch_validator


NETWORK_TIMEOUT_SECONDS = 30.0
DIRECTORY_FLAGS = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
READ_FLAGS = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
WRITE_FLAGS = os.O_RDWR | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_CREAT | os.O_EXCL
PUBLICATION_PROBE_SOURCE_NAME = "riotbox-1430-publication-probe-source"
PUBLICATION_PROBE_DESTINATION_NAME = "riotbox-1430-publication-probe-destination"


class RunnerError(RuntimeError):
    """Raised when the one-shot execution must stop fail-closed."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RunnerError(message)


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def safe_error_identifier(value: str) -> str:
    lowered = value.casefold()
    normalized = "".join(
        character if character.isascii() and (character.isalnum() or character in "_:.-") else "_"
        for character in lowered
    ).strip("_")
    return normalized[:96] or "unknown_error"


def _safe_relative_parts(value: str, prefix: str) -> tuple[str, ...]:
    require(isinstance(value, str) and value, f"{prefix} must be a non-empty path")
    require("\\" not in value and "\x00" not in value and "//" not in value, f"{prefix} contains a forbidden byte")
    path = PurePosixPath(value)
    require(
        not path.is_absolute()
        and path.parts
        and all(part not in {"", ".", ".."} for part in path.parts),
        f"{prefix} must be safe and repo-relative",
    )
    return path.parts


@dataclass
class DirectoryChain:
    repo_root: Path
    parts: tuple[str, ...]
    descriptors: list[int]
    identities: list[tuple[int, int]]

    @property
    def descriptor(self) -> int:
        return self.descriptors[-1]

    def revalidate(self) -> None:
        root = os.lstat(self.repo_root)
        require(
            stat.S_ISDIR(root.st_mode)
            and not stat.S_ISLNK(root.st_mode)
            and (root.st_dev, root.st_ino) == self.identities[0],
            "repository root namespace identity changed",
        )
        for index, part in enumerate(self.parts, start=1):
            named = os.stat(
                part,
                dir_fd=self.descriptors[index - 1],
                follow_symlinks=False,
            )
            opened = os.fstat(self.descriptors[index])
            require(
                stat.S_ISDIR(named.st_mode)
                and not stat.S_ISLNK(named.st_mode)
                and (named.st_dev, named.st_ino)
                == self.identities[index]
                == (opened.st_dev, opened.st_ino),
                f"directory namespace identity changed: {part}",
            )

    def close(self) -> None:
        while self.descriptors:
            os.close(self.descriptors.pop())


def _open_directory_chain(
    repo_root: Path,
    relative_directory: str,
    *,
    create_missing: bool,
) -> DirectoryChain:
    parts = _safe_relative_parts(relative_directory, "directory path")
    root_before = os.lstat(repo_root)
    require(stat.S_ISDIR(root_before.st_mode), "repository root is not a directory")
    require(not stat.S_ISLNK(root_before.st_mode), "repository root is a symlink")
    descriptors: list[int] = []
    identities: list[tuple[int, int]] = []
    try:
        root_fd = os.open(repo_root, DIRECTORY_FLAGS)
        descriptors.append(root_fd)
        root_opened = os.fstat(root_fd)
        require(
            (root_before.st_dev, root_before.st_ino)
            == (root_opened.st_dev, root_opened.st_ino),
            "repository root inode changed during open",
        )
        identities.append((root_opened.st_dev, root_opened.st_ino))
        for part in parts:
            parent_fd = descriptors[-1]
            try:
                before = os.stat(part, dir_fd=parent_fd, follow_symlinks=False)
            except FileNotFoundError:
                require(create_missing, f"required directory is missing: {part}")
                os.mkdir(part, 0o700, dir_fd=parent_fd)
                os.fsync(parent_fd)
                before = os.stat(part, dir_fd=parent_fd, follow_symlinks=False)
            require(not stat.S_ISLNK(before.st_mode), f"directory ancestor is a symlink: {part}")
            require(stat.S_ISDIR(before.st_mode), f"path ancestor is not a directory: {part}")
            child_fd = os.open(part, DIRECTORY_FLAGS, dir_fd=parent_fd)
            descriptors.append(child_fd)
            opened = os.fstat(child_fd)
            require(
                stat.S_ISDIR(opened.st_mode)
                and (before.st_dev, before.st_ino) == (opened.st_dev, opened.st_ino),
                f"directory ancestor changed during open: {part}",
            )
            identities.append((opened.st_dev, opened.st_ino))
        return DirectoryChain(repo_root, parts, descriptors, identities)
    except Exception:
        while descriptors:
            os.close(descriptors.pop())
        raise


def _split_parent(value: str, prefix: str) -> tuple[str, str]:
    parts = _safe_relative_parts(value, prefix)
    require(len(parts) >= 2, f"{prefix} must have a repository-relative parent")
    return "/".join(parts[:-1]), parts[-1]


def _assert_name_absent(parent_fd: int, name: str, prefix: str) -> None:
    try:
        os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    except FileNotFoundError:
        return
    raise RunnerError(f"{prefix} already exists; the one-shot attempt is consumed or unsafe")


def _discard_stale_access_log_update(parent_fd: int, name: str) -> None:
    """Remove only the fixed atomic-update name; the primary log stays authoritative."""

    try:
        before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    except FileNotFoundError:
        return
    require(
        stat.S_ISREG(before.st_mode) and before.st_nlink == 1,
        "stale access-log update is not a single-link regular file",
    )
    descriptor = os.open(
        name,
        os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
        dir_fd=parent_fd,
    )
    try:
        opened = os.fstat(descriptor)
        require(
            (before.st_dev, before.st_ino, before.st_nlink)
            == (opened.st_dev, opened.st_ino, opened.st_nlink),
            "stale access-log update changed during open",
        )
        os.unlink(name, dir_fd=parent_fd)
        os.fsync(parent_fd)
    finally:
        os.close(descriptor)


def _probe_publication_capability(
    source_parent_fd: int,
    destination_parent_fd: int,
    *,
    source_name: str,
    destination_name: str,
) -> None:
    """Exercise the exact cross-parent no-replace primitive using one empty dir."""

    _assert_name_absent(source_parent_fd, source_name, "publication probe source")
    _assert_name_absent(
        destination_parent_fd,
        destination_name,
        "publication probe destination",
    )
    moved = False
    os.mkdir(source_name, 0o700, dir_fd=source_parent_fd)
    try:
        os.fsync(source_parent_fd)
        acquisition.rename_directory_noreplace(
            source_parent_fd,
            source_name,
            destination_parent_fd,
            destination_name,
        )
        moved = True
        os.fsync(source_parent_fd)
        os.fsync(destination_parent_fd)
        published = os.stat(
            destination_name,
            dir_fd=destination_parent_fd,
            follow_symlinks=False,
        )
        require(
            stat.S_ISDIR(published.st_mode)
            and not stat.S_ISLNK(published.st_mode),
            "publication probe did not produce one exact directory",
        )
        os.rmdir(destination_name, dir_fd=destination_parent_fd)
        moved = False
        os.fsync(destination_parent_fd)
    finally:
        if moved:
            try:
                os.rmdir(destination_name, dir_fd=destination_parent_fd)
                os.fsync(destination_parent_fd)
            except FileNotFoundError:
                pass
        else:
            try:
                os.rmdir(source_name, dir_fd=source_parent_fd)
                os.fsync(source_parent_fd)
            except FileNotFoundError:
                pass


def _assert_parent_identity(parent_fd: int, expected: tuple[int, int], prefix: str) -> None:
    current = os.fstat(parent_fd)
    require(stat.S_ISDIR(current.st_mode), f"{prefix} is no longer a directory")
    require((current.st_dev, current.st_ino) == expected, f"{prefix} inode changed")


def _assert_directory_identity(
    directory_fd: int, expected: tuple[int, int], prefix: str
) -> os.stat_result:
    current = os.fstat(directory_fd)
    require(stat.S_ISDIR(current.st_mode), f"{prefix} is not a directory")
    require((current.st_dev, current.st_ino) == expected, f"{prefix} inode changed")
    return current


def _assert_named_directory_identity(
    parent_fd: int,
    name: str,
    directory_fd: int,
    expected: tuple[int, int],
    prefix: str,
) -> os.stat_result:
    named = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    opened = _assert_directory_identity(directory_fd, expected, prefix)
    require(
        stat.S_ISDIR(named.st_mode)
        and not stat.S_ISLNK(named.st_mode)
        and (named.st_dev, named.st_ino) == expected,
        f"{prefix} name no longer resolves to the held directory",
    )
    return opened


def _assert_named_regular_identity(
    parent_fd: int, name: str, descriptor: int, prefix: str
) -> None:
    named = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    opened = os.fstat(descriptor)
    require(
        stat.S_ISREG(named.st_mode)
        and not stat.S_ISLNK(named.st_mode)
        and named.st_nlink == 1
        and (
            named.st_dev,
            named.st_ino,
            named.st_nlink,
        )
        == (
            opened.st_dev,
            opened.st_ino,
            opened.st_nlink,
        ),
        f"{prefix} name no longer resolves to the held file",
    )


def _pwrite_all(descriptor: int, payload: bytes) -> None:
    offset = 0
    while offset < len(payload):
        written = os.pwrite(descriptor, payload[offset:], offset)
        require(written > 0, "durable JSON write made no progress")
        offset += written


def _persist_json_fd(descriptor: int, document: dict[str, Any]) -> bytes:
    payload = artifacts.render(document)
    before = os.fstat(descriptor)
    require(stat.S_ISREG(before.st_mode), "artifact descriptor is not a regular file")
    require(before.st_nlink == 1, "artifact descriptor must have one hard link")
    _pwrite_all(descriptor, payload)
    os.ftruncate(descriptor, len(payload))
    os.fsync(descriptor)
    after = os.fstat(descriptor)
    require(
        (before.st_dev, before.st_ino, before.st_nlink)
        == (after.st_dev, after.st_ino, after.st_nlink)
        and after.st_size == len(payload),
        "artifact inode changed during durable write",
    )
    return payload


def _persist_access_log_atomically(
    *,
    parent_fd: int,
    current_fd: int,
    current_name: str,
    update_name: str,
    document: dict[str, Any],
) -> tuple[int, bytes]:
    """Replace a valid log with a fully fsynced successor, never torn in-place."""

    current_by_name = os.stat(
        current_name, dir_fd=parent_fd, follow_symlinks=False
    )
    current_opened = os.fstat(current_fd)
    require(
        stat.S_ISREG(current_by_name.st_mode)
        and current_by_name.st_nlink == 1
        and (
            current_by_name.st_dev,
            current_by_name.st_ino,
            current_by_name.st_nlink,
        )
        == (
            current_opened.st_dev,
            current_opened.st_ino,
            current_opened.st_nlink,
        ),
        "access-log name no longer resolves to the held current inode",
    )
    _assert_name_absent(parent_fd, update_name, "access-log atomic update")
    update_fd: int | None = None
    replaced = False
    try:
        update_fd = _open_exclusive_regular(parent_fd, update_name, 0o600)
        payload = _persist_json_fd(update_fd, document)
        update_stat = os.fstat(update_fd)
        os.replace(
            update_name,
            current_name,
            src_dir_fd=parent_fd,
            dst_dir_fd=parent_fd,
        )
        replaced = True
        os.fsync(parent_fd)
        published = os.stat(
            current_name, dir_fd=parent_fd, follow_symlinks=False
        )
        require(
            (
                published.st_dev,
                published.st_ino,
                published.st_size,
                published.st_nlink,
            )
            == (
                update_stat.st_dev,
                update_stat.st_ino,
                update_stat.st_size,
                update_stat.st_nlink,
            ),
            "atomic access-log successor identity changed",
        )
        os.close(current_fd)
        return update_fd, payload
    except Exception:
        if update_fd is not None:
            os.close(update_fd)
        if not replaced:
            try:
                os.unlink(update_name, dir_fd=parent_fd)
                os.fsync(parent_fd)
            except FileNotFoundError:
                pass
        raise


def _open_exclusive_regular(parent_fd: int, name: str, mode: int) -> int:
    descriptor = os.open(name, WRITE_FLAGS, mode, dir_fd=parent_fd)
    opened = os.fstat(descriptor)
    require(stat.S_ISREG(opened.st_mode), "exclusive artifact is not a regular file")
    require(opened.st_nlink == 1, "exclusive artifact has an unsafe hard-link count")
    return descriptor


def _open_existing_regular(parent_fd: int, name: str, *, writable: bool) -> int:
    flags = (os.O_RDWR if writable else os.O_RDONLY) | os.O_CLOEXEC | os.O_NOFOLLOW
    before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    require(stat.S_ISREG(before.st_mode), f"existing artifact is not regular: {name}")
    require(before.st_nlink == 1, f"existing artifact has unsafe hard-link count: {name}")
    descriptor = os.open(name, flags, dir_fd=parent_fd)
    opened = os.fstat(descriptor)
    require(
        (before.st_dev, before.st_ino, before.st_size, before.st_nlink)
        == (opened.st_dev, opened.st_ino, opened.st_size, opened.st_nlink),
        f"existing artifact changed during open: {name}",
    )
    return descriptor


def _read_bounded_fd(descriptor: int, maximum_bytes: int, prefix: str) -> bytes:
    before = os.fstat(descriptor)
    require(before.st_size <= maximum_bytes, f"{prefix} exceeds its byte bound")
    chunks: list[bytes] = []
    offset = 0
    while offset < before.st_size:
        chunk = os.pread(descriptor, min(65_536, before.st_size - offset), offset)
        require(chunk, f"{prefix} truncated during read")
        chunks.append(chunk)
        offset += len(chunk)
    after = os.fstat(descriptor)
    require(
        (
            before.st_dev,
            before.st_ino,
            before.st_mode,
            before.st_size,
            before.st_nlink,
            before.st_mtime_ns,
            before.st_ctime_ns,
        )
        == (
            after.st_dev,
            after.st_ino,
            after.st_mode,
            after.st_size,
            after.st_nlink,
            after.st_mtime_ns,
            after.st_ctime_ns,
        ),
        f"{prefix} changed during bounded read",
    )
    return b"".join(chunks)


@dataclass(frozen=True)
class ResolvedEndpoint:
    family: int
    sockaddr: tuple[Any, ...]
    canonical_ip: str


def _resolve_one_public_endpoint(host: str) -> tuple[list[str], ResolvedEndpoint]:
    require(host == acquisition.PROVIDER_HOST, "network host differs from frozen provider")
    answers = socket.getaddrinfo(
        f"{host}.",
        443,
        family=socket.AF_UNSPEC,
        type=socket.SOCK_STREAM,
        proto=socket.IPPROTO_TCP,
        flags=0,
    )
    require(answers, "DNS returned no addresses")
    endpoints: dict[tuple[int, str], ResolvedEndpoint] = {}
    for family, socket_type, protocol, _canonical_name, sockaddr in answers:
        require(family in {socket.AF_INET, socket.AF_INET6}, "DNS returned an unsupported address family")
        require(socket_type == socket.SOCK_STREAM, "DNS returned a non-stream socket")
        require(protocol in {0, socket.IPPROTO_TCP}, "DNS returned a non-TCP protocol")
        require(isinstance(sockaddr, tuple) and len(sockaddr) >= 2 and sockaddr[1] == 443, "DNS returned an unexpected port")
        canonical = acquisition.validate_public_peer_ip(sockaddr[0], "DNS answer")
        endpoints[(family, canonical)] = ResolvedEndpoint(family, sockaddr, canonical)
    ordered = sorted(endpoints.values(), key=lambda item: (item.family, item.canonical_ip))
    require(ordered, "DNS returned no usable public addresses")
    logged_addresses = sorted({item.canonical_ip for item in ordered})
    return logged_addresses, ordered[0]


@dataclass
class ExactHttpResponse:
    response: http.client.HTTPResponse
    tls_socket: ssl.SSLSocket
    connected_peer_ip: str
    negotiated_tls_version: str
    peer_certificate_sha256: str

    def close(self) -> None:
        try:
            self.response.close()
        finally:
            self.tls_socket.close()


def _open_exact_https_response(
    endpoint: ResolvedEndpoint,
    request_target: str,
) -> ExactHttpResponse:
    require(request_target.startswith("/") and "?" not in request_target and "#" not in request_target, "request target must be one exact absolute path")
    try:
        encoded_target = request_target.encode("ascii")
    except UnicodeEncodeError as error:
        raise RunnerError("request target must be ASCII") from error
    require(
        all(0x21 <= byte <= 0x7E for byte in encoded_target),
        "request target contains an HTTP control or whitespace byte",
    )
    raw_socket = socket.socket(endpoint.family, socket.SOCK_STREAM, socket.IPPROTO_TCP)
    raw_socket.settimeout(NETWORK_TIMEOUT_SECONDS)
    tls_socket: ssl.SSLSocket | None = None
    try:
        raw_socket.connect(endpoint.sockaddr)
        connected = acquisition.validate_public_peer_ip(
            raw_socket.getpeername()[0], "connected peer"
        )
        require(connected == endpoint.canonical_ip, "connected peer differs from selected DNS address")
        context = ssl.create_default_context()
        context.minimum_version = ssl.TLSVersion.TLSv1_2
        context.check_hostname = True
        context.verify_mode = ssl.CERT_REQUIRED
        tls_socket = context.wrap_socket(
            raw_socket,
            server_hostname=acquisition.PROVIDER_HOST,
        )
        connected_after_tls = acquisition.validate_public_peer_ip(
            tls_socket.getpeername()[0], "TLS peer"
        )
        require(connected_after_tls == endpoint.canonical_ip, "TLS peer differs from selected DNS address")
        negotiated = tls_socket.version()
        require(negotiated in {"TLSv1.2", "TLSv1.3"}, "TLS negotiation fell below the frozen minimum")
        certificate = tls_socket.getpeercert(binary_form=True)
        require(isinstance(certificate, bytes) and certificate, "TLS peer certificate is missing")
        request_lines = [f"GET {request_target} HTTP/1.1"] + [
            f"{name}: {value}" for name, value in artifacts.REQUEST_HEADERS
        ]
        request_payload = ("\r\n".join(request_lines) + "\r\n\r\n").encode("ascii")
        tls_socket.sendall(request_payload)
        response = http.client.HTTPResponse(tls_socket, method="GET")
        response.begin()
        return ExactHttpResponse(
            response=response,
            tls_socket=tls_socket,
            connected_peer_ip=connected_after_tls,
            negotiated_tls_version=negotiated,
            peer_certificate_sha256=hashlib.sha256(certificate).hexdigest(),
        )
    except Exception:
        if tls_socket is not None:
            tls_socket.close()
        else:
            raw_socket.close()
        raise


def _raw_header_map(response: http.client.HTTPResponse) -> dict[str, list[str]]:
    headers: dict[str, list[str]] = {}
    for name, value in response.getheaders():
        require(isinstance(name, str) and isinstance(value, str), "HTTP response header is not textual")
        require("\r" not in name and "\n" not in name and "\r" not in value and "\n" not in value, "HTTP response header contains a control newline")
        headers.setdefault(name.casefold(), []).append(value)
    return headers


def _transition(log: dict[str, Any], state: str, at_utc: str) -> None:
    log["attempt_status"] = state
    log["transition_history"].append(
        {
            "sequence": len(log["transition_history"]) + 1,
            "state": state,
            "at_utc": at_utc,
        }
    )


def _manifest_for_log(
    log: dict[str, Any],
    manifest_document: dict[str, Any] | None,
    manifest_payload: bytes | None,
) -> tuple[dict[str, Any] | None, bytes | None]:
    if log["sealed_manifest"]["state"] in {"sealed_in_quarantine", "published"}:
        require(manifest_document is not None and manifest_payload is not None, "sealed log lost its in-memory manifest")
        return manifest_document, manifest_payload
    return None, None


def _validate_log_before_persist(
    log: dict[str, Any],
    *,
    frozen: batch_validator.FrozenAcquisitionBatchV1,
    repo_root: Path,
    manifest_document: dict[str, Any] | None,
    manifest_payload: bytes | None,
) -> None:
    validated_manifest, validated_payload = _manifest_for_log(
        log, manifest_document, manifest_payload
    )
    artifact_validator.validate_access_log_document(
        log,
        frozen_batch=frozen,
        repo_root=repo_root,
        manifest_document=validated_manifest,
        manifest_payload=validated_payload,
    )


def _pass_checkpoint(
    log: dict[str, Any],
    checkpoint_index: int,
    *,
    frozen: batch_validator.FrozenAcquisitionBatchV1,
    repo_root: Path,
) -> None:
    revalidated = batch_validator.revalidate_frozen_batch(frozen)
    current_snapshot = artifacts.build_implementation_snapshot(repo_root)
    require(
        current_snapshot["algorithm"] == log["implementation_snapshot"]["algorithm"]
        and current_snapshot["files"] == log["implementation_snapshot"]["files"]
        and current_snapshot["aggregate_sha256"]
        == log["implementation_snapshot"]["aggregate_sha256"],
        "acquisition implementation bytes changed",
    )
    record = log["revalidation_checkpoints"][checkpoint_index]
    require(record["status"] == "pending", "revalidation checkpoint was reused")
    record["status"] = "passed"
    record["checked_at_utc"] = utc_now()
    record["observed_bindings"] = artifacts.observed_bindings(
        revalidated.raw_sha256,
        revalidated.semantic_sha256,
        current_snapshot["aggregate_sha256"],
    )


def _registry_v2_payload_hashes(repo_root: Path) -> set[str]:
    path = repo_root / batch_contract.REGISTRY_V2_REL
    document = artifact_validator.parse_json(
        artifacts._read_regular_file_no_follow(path),
        str(batch_contract.REGISTRY_V2_REL),
    )
    entries = document.get("entries")
    require(isinstance(entries, list), "Registry-v2 entries are missing")
    hashes: set[str] = set()
    for entry in entries:
        require(isinstance(entry, dict), "Registry-v2 entry is not an object")
        digest = entry.get("sha256")
        require(
            isinstance(digest, str)
            and artifact_validator.SHA256.fullmatch(digest) is not None,
            "Registry-v2 payload hash is malformed",
        )
        hashes.add(digest)
    require(hashes, "Registry-v2 payload identity set is empty")
    return hashes


def _assert_unique_payload_hash(
    digest: str,
    *,
    registry_v2_hashes: set[str],
    new_hashes: set[str],
) -> None:
    require(digest not in registry_v2_hashes, "payload_identity_collision_registry_v2")
    require(digest not in new_hashes, "payload_identity_collision_within_new_batch")


def _verify_download_file(
    descriptor: int,
    *,
    expected_identity: tuple[int, int],
    expected_size: int,
    prefix: str,
) -> None:
    current = os.fstat(descriptor)
    require(stat.S_ISREG(current.st_mode), f"{prefix} is not a regular file")
    require(current.st_nlink == 1, f"{prefix} hard-link count changed")
    require(
        (current.st_dev, current.st_ino) == expected_identity,
        f"{prefix} inode changed",
    )
    require(current.st_size == expected_size, f"{prefix} size changed")


def _cleanup_quarantine(
    *,
    quarantine_fd: int,
    quarantine_parent_fd: int,
    quarantine_name: str,
    created_names: list[str],
) -> str:
    try:
        os.fchmod(quarantine_fd, 0o700)
        for name in reversed(created_names):
            try:
                os.unlink(name, dir_fd=quarantine_fd)
            except FileNotFoundError:
                pass
        os.fsync(quarantine_fd)
        os.close(quarantine_fd)
        os.rmdir(quarantine_name, dir_fd=quarantine_parent_fd)
        os.fsync(quarantine_parent_fd)
        return "removed_exact_known_names"
    except OSError:
        try:
            os.close(quarantine_fd)
        except OSError:
            pass
        return "cleanup_incomplete_fail_closed"


def _append_rejection(
    log: dict[str, Any],
    *,
    stage: str,
    reason_code: str,
    error: Exception,
    publication_authorized: bool,
) -> None:
    at_utc = utc_now()
    _transition(log, "rejected", at_utc)
    log["completed_at_utc"] = at_utc
    log["rejection"] = {
        "at_utc": at_utc,
        "stage": safe_error_identifier(stage),
        "reason_code": safe_error_identifier(reason_code),
        "error_type": safe_error_identifier(type(error).__name__),
        "error_message_sha256": hashlib.sha256(str(error).encode("utf-8")).hexdigest(),
        "requests_started": log["request_count"],
        "successful_requests": log["successful_request_count"],
        "further_requests_performed": False,
        "publication_authorized": publication_authorized,
        "new_versioned_metadata_decision_required": True,
    }


def run_acquisition(repo_root: Path) -> tuple[str, str]:
    """Run the frozen three-request transaction or stop permanently on failure."""

    frozen = batch_validator.validate_repository(repo_root)
    batch = frozen.document
    implementation_snapshot = artifacts.build_implementation_snapshot(repo_root)
    registry_v2_hashes = _registry_v2_payload_hashes(repo_root)
    observed_new_hashes: set[str] = set()

    log_parent_rel, log_name = _split_parent(
        batch["filesystem_contract"]["access_log_path"], "access log path"
    )
    log_update_rel, log_update_name = _split_parent(
        batch["filesystem_contract"]["access_log_path"]
        + artifacts.ACCESS_LOG_UPDATE_SUFFIX,
        "access log atomic update path",
    )
    quarantine_parent_rel, quarantine_name = _split_parent(
        batch["filesystem_contract"]["quarantine_directory"],
        "quarantine directory",
    )
    final_parent_rel, final_name = _split_parent(
        batch["filesystem_contract"]["final_batch_directory"],
        "final batch directory",
    )
    require(
        log_parent_rel == log_update_rel == quarantine_parent_rel,
        "access log, atomic update, and quarantine must share the exact declared parent",
    )
    log_parent_chain = _open_directory_chain(
        repo_root, log_parent_rel, create_missing=True
    )
    final_parent_chain = _open_directory_chain(
        repo_root, final_parent_rel, create_missing=False
    )
    log_parent_fd = log_parent_chain.descriptor
    final_parent_fd = final_parent_chain.descriptor
    log_parent_stat = os.fstat(log_parent_fd)
    final_parent_stat = os.fstat(final_parent_fd)
    log_parent_identity = (log_parent_stat.st_dev, log_parent_stat.st_ino)
    final_parent_identity = (final_parent_stat.st_dev, final_parent_stat.st_ino)
    require(
        log_parent_stat.st_dev == final_parent_stat.st_dev,
        "quarantine and final parents are on different filesystems",
    )
    _assert_name_absent(log_parent_fd, log_name, "access log")
    _assert_name_absent(log_parent_fd, log_update_name, "access-log atomic update")
    _assert_name_absent(log_parent_fd, quarantine_name, "quarantine directory")
    _assert_name_absent(final_parent_fd, final_name, "final batch directory")
    required_free_bytes = sum(
        entry["attachment_byte_count"] for entry in batch["entries"]
    ) + 8_388_608
    filesystem_capacity = os.fstatvfs(final_parent_fd)
    available_free_bytes = (
        filesystem_capacity.f_bavail * filesystem_capacity.f_frsize
    )
    require(
        available_free_bytes >= required_free_bytes,
        "final filesystem lacks the frozen preflight free-space requirement",
    )
    _probe_publication_capability(
        log_parent_fd,
        final_parent_fd,
        source_name=PUBLICATION_PROBE_SOURCE_NAME,
        destination_name=PUBLICATION_PROBE_DESTINATION_NAME,
    )
    log_parent_chain.revalidate()
    final_parent_chain.revalidate()
    _assert_name_absent(
        log_parent_fd,
        PUBLICATION_PROBE_SOURCE_NAME,
        "publication probe source",
    )
    _assert_name_absent(
        final_parent_fd,
        PUBLICATION_PROBE_DESTINATION_NAME,
        "publication probe destination",
    )
    _assert_name_absent(final_parent_fd, final_name, "final batch directory")

    log_fd: int | None = None
    quarantine_fd: int | None = None
    quarantine_identity: tuple[int, int] | None = None
    created_names: list[str] = []
    manifest_document: dict[str, Any] | None = None
    manifest_payload: bytes | None = None
    log: dict[str, Any] | None = None
    durable_log: dict[str, Any] | None = None
    durable_log_payload: bytes | None = None
    stage = "initialization"
    reason_code = "execution_failure"
    renamed = False
    log_initialized = False

    def persist() -> None:
        nonlocal durable_log, durable_log_payload, log_fd, log_initialized
        require(log is not None and log_fd is not None, "access log is unavailable")
        _validate_log_before_persist(
            log,
            frozen=frozen,
            repo_root=repo_root,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        if log_initialized:
            log_fd, durable_log_payload = _persist_access_log_atomically(
                parent_fd=log_parent_fd,
                current_fd=log_fd,
                current_name=log_name,
                update_name=log_update_name,
                document=log,
            )
        else:
            durable_log_payload = _persist_json_fd(log_fd, log)
            log_initialized = True
        durable_log = artifacts.clone(log)

    try:
        stage = "access_log_creation"
        log_fd = _open_exclusive_regular(log_parent_fd, log_name, 0o600)
        started_at = utc_now()
        log = artifacts.build_initial_access_log(
            batch_document=batch,
            batch_raw_sha256=frozen.raw_sha256,
            batch_semantic_sha256=frozen.semantic_sha256,
            implementation_snapshot=implementation_snapshot,
            attempt_id=str(uuid.uuid4()),
            started_at_utc=started_at,
        )
        log["filesystem"]["publication_probe_completed"] = True
        log["filesystem"]["available_free_bytes_before_network"] = (
            available_free_bytes
        )
        persist()
        os.fsync(log_parent_fd)
        log["filesystem"]["access_log_parent_fsync_completed"] = True
        persist()

        stage = "quarantine_creation"
        os.mkdir(quarantine_name, 0o700, dir_fd=log_parent_fd)
        os.fsync(log_parent_fd)
        quarantine_before = os.stat(
            quarantine_name, dir_fd=log_parent_fd, follow_symlinks=False
        )
        require(
            stat.S_ISDIR(quarantine_before.st_mode)
            and not stat.S_ISLNK(quarantine_before.st_mode),
            "quarantine is not a no-follow directory",
        )
        quarantine_fd = os.open(quarantine_name, DIRECTORY_FLAGS, dir_fd=log_parent_fd)
        quarantine_opened = os.fstat(quarantine_fd)
        require(
            (quarantine_before.st_dev, quarantine_before.st_ino)
            == (quarantine_opened.st_dev, quarantine_opened.st_ino),
            "quarantine inode changed during open",
        )
        quarantine_identity = (
            quarantine_opened.st_dev,
            quarantine_opened.st_ino,
        )
        require(
            quarantine_opened.st_dev == final_parent_stat.st_dev,
            "quarantine directory is not on the final filesystem",
        )
        log["filesystem"]["quarantine_created_exclusively"] = True
        _transition(log, "preflight_passed", utc_now())
        persist()

        for index, (entry, record) in enumerate(
            zip(batch["entries"], log["entries"], strict=True)
        ):
            stage = f"checkpoint_before_request_{index + 1}"
            reason_code = "contract_or_implementation_drift"
            log_parent_chain.revalidate()
            final_parent_chain.revalidate()
            _assert_parent_identity(log_parent_fd, log_parent_identity, "quarantine parent")
            _assert_parent_identity(final_parent_fd, final_parent_identity, "final parent")
            require(quarantine_fd is not None and quarantine_identity is not None, "quarantine descriptor is unavailable")
            _assert_named_directory_identity(
                log_parent_fd,
                quarantine_name,
                quarantine_fd,
                quarantine_identity,
                "quarantine",
            )
            require(log_fd is not None, "access-log descriptor is unavailable")
            _assert_named_regular_identity(
                log_parent_fd, log_name, log_fd, "access log"
            )
            _assert_name_absent(final_parent_fd, final_name, "final batch directory")
            _pass_checkpoint(log, index, frozen=frozen, repo_root=repo_root)
            persist()

            if index == 0:
                _transition(log, "acquiring", utc_now())
            stage = f"request_{index + 1}_reservation"
            reason_code = "request_failed"
            request_started_at = utc_now()
            record["state"] = "request_started"
            record["request_count"] = 1
            record["request_started_at_utc"] = request_started_at
            log["request_count"] += 1
            persist()

            stage = f"request_{index + 1}_dns"
            dns_answers, endpoint = _resolve_one_public_endpoint(
                acquisition.PROVIDER_HOST
            )
            record["network"]["dns_answers"] = dns_answers
            record["network"]["selected_peer_ip"] = endpoint.canonical_ip
            persist()

            response_session: ExactHttpResponse | None = None
            destination_fd: int | None = None
            try:
                stage = f"request_{index + 1}_https"
                response_session = _open_exact_https_response(
                    endpoint, record["network"]["request_target"]
                )
                record["network"]["connected_peer_ip"] = (
                    response_session.connected_peer_ip
                )
                record["network"]["tls"]["negotiated_version"] = (
                    response_session.negotiated_tls_version
                )
                record["network"]["tls"]["peer_certificate_sha256"] = (
                    response_session.peer_certificate_sha256
                )
                headers = _raw_header_map(response_session.response)
                response_gate = acquisition.validate_response_gate(
                    status=response_session.response.status,
                    headers=headers,
                    expected_byte_count=entry["attachment_byte_count"],
                )
                record["network"]["response_gate"] = response_gate
                record["state"] = "response_metadata_verified"
                persist()

                stage = f"request_{index + 1}_body"
                destination_name = PurePosixPath(entry["destination_path"]).name
                destination_fd = _open_exclusive_regular(
                    quarantine_fd, destination_name, 0o600
                )
                created_names.append(destination_name)
                destination_stat = os.fstat(destination_fd)
                destination_identity = (
                    destination_stat.st_dev,
                    destination_stat.st_ino,
                )
                stream_record = acquisition.stream_exact_attachment(
                    response_session.response,
                    destination_fd,
                    entry["attachment_byte_count"],
                )
                _assert_unique_payload_hash(
                    stream_record["actual_sha256"],
                    registry_v2_hashes=registry_v2_hashes,
                    new_hashes=observed_new_hashes,
                )
                _verify_download_file(
                    destination_fd,
                    expected_identity=destination_identity,
                    expected_size=entry["attachment_byte_count"],
                    prefix=f"entry {index + 1} quarantine file",
                )
                record["stream"] = stream_record
                record["quarantine_file_identity"] = {
                    "device": destination_stat.st_dev,
                    "inode": destination_stat.st_ino,
                    "link_count": 1,
                    "byte_count": entry["attachment_byte_count"],
                }
                record["state"] = "body_verified"
                persist()

                stage = f"request_{index + 1}_header"
                header_record = acquisition.inspect_riff_pcm_header_only(
                    destination_fd, entry["attachment_byte_count"]
                )
                _verify_download_file(
                    destination_fd,
                    expected_identity=destination_identity,
                    expected_size=entry["attachment_byte_count"],
                    prefix=f"entry {index + 1} quarantine file",
                )
                os.fchmod(destination_fd, 0o400)
                os.fsync(destination_fd)
                record["header_validation"] = header_record
                record["state"] = "header_verified"
                record["verified_at_utc"] = utc_now()
                log["successful_request_count"] += 1
                observed_new_hashes.add(stream_record["actual_sha256"])
                persist()
            finally:
                if destination_fd is not None:
                    os.close(destination_fd)
                if response_session is not None:
                    response_session.close()

        stage = "all_headers_verified"
        reason_code = "sealing_failed"
        _transition(log, "all_headers_verified", utc_now())
        persist()

        stage = "manifest_sealing"
        manifest_document = artifacts.build_sealed_manifest(log)
        artifact_validator.validate_manifest_document(
            manifest_document,
            access_log=log,
            frozen_batch=frozen,
            repo_root=repo_root,
        )
        manifest_payload = artifacts.render(manifest_document)
        manifest_name = batch["filesystem_contract"]["sealed_manifest_name"]
        manifest_fd = _open_exclusive_regular(quarantine_fd, manifest_name, 0o600)
        created_names.append(manifest_name)
        manifest_identity: os.stat_result | None = None
        try:
            written_payload = _persist_json_fd(manifest_fd, manifest_document)
            require(written_payload == manifest_payload, "manifest renderer changed during sealing")
            os.fchmod(manifest_fd, 0o400)
            os.fsync(manifest_fd)
            manifest_identity = os.fstat(manifest_fd)
        finally:
            os.close(manifest_fd)
        require(manifest_identity is not None, "sealed manifest identity is unavailable")
        os.fchmod(quarantine_fd, 0o500)
        os.fsync(quarantine_fd)
        for record in log["entries"]:
            record["state"] = "sealed"
        log["sealed_manifest"].update(
            {
                "state": "sealed_in_quarantine",
                "byte_count": len(manifest_payload),
                "raw_sha256": hashlib.sha256(manifest_payload).hexdigest(),
                "semantic_sha256": artifacts.semantic_sha256(manifest_document),
                "quarantine_file_identity": {
                    "device": manifest_identity.st_dev,
                    "inode": manifest_identity.st_ino,
                    "link_count": manifest_identity.st_nlink,
                    "byte_count": manifest_identity.st_size,
                },
            }
        )
        _transition(log, "sealed_in_quarantine", utc_now())
        persist()

        stage = "sealed_payload_revalidation"
        reason_code = "sealed_payload_identity_or_header_drift"
        artifact_validator.validate_sealed_directory_fd(
            quarantine_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        log["sealed_payload_revalidation"].update(
            {
                "state": "passed",
                "checked_at_utc": utc_now(),
                "entry_count": 3,
                "raw_sha256_recomputed": True,
                "header_reinspection_performed": True,
            }
        )
        persist()

        stage = "checkpoint_before_publication"
        reason_code = "publication_precondition_failed"
        _pass_checkpoint(log, 3, frozen=frozen, repo_root=repo_root)
        log_parent_chain.revalidate()
        final_parent_chain.revalidate()
        _assert_parent_identity(log_parent_fd, log_parent_identity, "quarantine parent")
        _assert_parent_identity(final_parent_fd, final_parent_identity, "final parent")
        _assert_name_absent(final_parent_fd, final_name, "final batch directory")
        prepared = _assert_named_directory_identity(
            log_parent_fd,
            quarantine_name,
            quarantine_fd,
            quarantine_identity,
            "sealed quarantine",
        )
        os.fsync(quarantine_fd)
        log["publication"]["state"] = "pending"
        log["publication"]["prepared_directory_device"] = prepared.st_dev
        log["publication"]["prepared_directory_inode"] = prepared.st_ino
        _transition(log, "publication_pending", utc_now())
        persist()

        stage = "atomic_publication"
        reason_code = "atomic_publication_failed"
        log_parent_chain.revalidate()
        final_parent_chain.revalidate()
        _assert_named_directory_identity(
            log_parent_fd,
            quarantine_name,
            quarantine_fd,
            quarantine_identity,
            "publication-pending quarantine",
        )
        _assert_named_regular_identity(
            log_parent_fd, log_name, log_fd, "publication-pending access log"
        )
        _assert_name_absent(final_parent_fd, final_name, "final batch directory")
        artifact_validator.validate_sealed_directory_fd(
            quarantine_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        # Linux requires write permission on a moved directory when its `..`
        # entry changes across parents. Open only this minimal rename window;
        # the exact sealed bytes were just checked and are checked again after.
        os.fchmod(quarantine_fd, 0o700)
        acquisition.rename_directory_noreplace(
            log_parent_fd,
            quarantine_name,
            final_parent_fd,
            final_name,
        )
        renamed = True
        os.fchmod(quarantine_fd, 0o500)
        os.fsync(quarantine_fd)
        os.fsync(log_parent_fd)
        os.fsync(final_parent_fd)
        published = os.stat(final_name, dir_fd=final_parent_fd, follow_symlinks=False)
        require(
            stat.S_ISDIR(published.st_mode)
            and (published.st_dev, published.st_ino) == quarantine_identity,
            "published directory identity differs from sealed quarantine",
        )
        try:
            os.stat(quarantine_name, dir_fd=log_parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            pass
        else:
            raise RunnerError("quarantine name survived atomic publication")
        artifact_validator.validate_sealed_directory_fd(
            quarantine_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        published_at = utc_now()
        for record in log["entries"]:
            record["state"] = "published"
        log["sealed_manifest"]["state"] = "published"
        log["publication"].update(
            {
                "state": "completed",
                "rename_count": 1,
                "source_parent_fsync_completed": True,
                "destination_parent_fsync_completed": True,
                "published_at_utc": published_at,
                "published_directory_device": published.st_dev,
                "published_directory_inode": published.st_ino,
            }
        )
        _transition(log, "completed", published_at)
        log["completed_at_utc"] = published_at
        persist()
        require(durable_log_payload is not None and manifest_payload is not None, "terminal artifacts are missing")
        log_parent_chain.revalidate()
        final_parent_chain.revalidate()
        _assert_named_regular_identity(
            log_parent_fd, log_name, log_fd, "completed access log"
        )
        _assert_named_directory_identity(
            final_parent_fd,
            final_name,
            quarantine_fd,
            quarantine_identity,
            "completed final batch",
        )
        artifact_validator.validate_sealed_directory_fd(
            quarantine_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        terminal_hashes = artifact_validator.validate_repository_terminal(repo_root)
        require(
            terminal_hashes
            == (
                hashlib.sha256(durable_log_payload).hexdigest(),
                hashlib.sha256(manifest_payload).hexdigest(),
            ),
            "terminal namespace hashes differ from the durable artifacts",
        )
        return terminal_hashes
    except Exception as error:
        if renamed:
            if durable_log is not None and durable_log.get("attempt_status") == "completed":
                raise RunnerError(
                    "terminal completed log is durable; run the terminal validator "
                    "without any network access"
                ) from error
            raise RunnerError(
                "atomic directory rename completed but terminal durability did not; "
                "the durable publication_pending log was preserved and only the "
                "no-network reconciler may continue"
            ) from error
        if durable_log is not None:
            log = artifacts.clone(durable_log)
        if log is not None and log.get("attempt_status") != "completed":
            if quarantine_fd is not None:
                log["filesystem"]["quarantine_cleanup_state"] = _cleanup_quarantine(
                    quarantine_fd=quarantine_fd,
                    quarantine_parent_fd=log_parent_fd,
                    quarantine_name=quarantine_name,
                    created_names=created_names,
                )
                quarantine_fd = None
            _append_rejection(
                log,
                stage=stage,
                reason_code=reason_code,
                error=error,
                publication_authorized=False,
            )
            if log_fd is not None:
                try:
                    persist()
                except Exception as logging_error:
                    raise RunnerError(
                        "acquisition failed and its existing exclusive log could not be finalized; "
                        "the consumed attempt remains fail-closed"
                    ) from logging_error
        raise
    finally:
        if quarantine_fd is not None:
            os.close(quarantine_fd)
        if log_fd is not None:
            os.close(log_fd)
        final_parent_chain.close()
        log_parent_chain.close()


def reconcile_publication_without_network(repo_root: Path) -> tuple[str, str]:
    """Finish only an already durable publication intent; never issue a request."""

    frozen = batch_validator.validate_repository(repo_root)
    batch = frozen.document
    log_parent_rel, log_name = _split_parent(
        batch["filesystem_contract"]["access_log_path"], "access log path"
    )
    log_update_rel, log_update_name = _split_parent(
        batch["filesystem_contract"]["access_log_path"]
        + artifacts.ACCESS_LOG_UPDATE_SUFFIX,
        "access log atomic update path",
    )
    quarantine_parent_rel, quarantine_name = _split_parent(
        batch["filesystem_contract"]["quarantine_directory"],
        "quarantine directory",
    )
    final_parent_rel, final_name = _split_parent(
        batch["filesystem_contract"]["final_batch_directory"],
        "final batch directory",
    )
    require(
        log_parent_rel == log_update_rel == quarantine_parent_rel,
        "reconciliation parent mismatch",
    )
    log_parent_chain = _open_directory_chain(
        repo_root, log_parent_rel, create_missing=False
    )
    final_parent_chain = _open_directory_chain(
        repo_root, final_parent_rel, create_missing=False
    )
    log_fd: int | None = None
    batch_directory_fd: int | None = None
    try:
        log_parent_fd = log_parent_chain.descriptor
        final_parent_fd = final_parent_chain.descriptor
        require(
            os.fstat(log_parent_fd).st_dev == os.fstat(final_parent_fd).st_dev,
            "reconciliation parents are on different filesystems",
        )
        _discard_stale_access_log_update(log_parent_fd, log_update_name)
        log_fd = _open_existing_regular(log_parent_fd, log_name, writable=True)
        log_parent_chain.revalidate()
        final_parent_chain.revalidate()
        _assert_named_regular_identity(
            log_parent_fd, log_name, log_fd, "reconciliation access log"
        )
        log_payload = _read_bounded_fd(log_fd, 2_097_152, "access log")
        log = artifact_validator.parse_json(log_payload, batch_contract.ACCESS_LOG_PATH)
        require(log_payload == artifacts.render(log), "access log bytes are not deterministic")
        if log.get("attempt_status") == "completed":
            # A prior atomic successor may have reached rename but failed while
            # fsyncing this directory. Reconciliation makes that visible name
            # durable before accepting the already-complete state.
            os.fsync(log_parent_fd)
            return artifact_validator.validate_repository_terminal(repo_root)
        require(
            log.get("attempt_status") == "publication_pending"
            and isinstance(log.get("publication"), dict)
            and log["publication"].get("state") == "pending"
            and log["publication"].get("rename_count") == 0,
            "only a durable publication_pending attempt can be reconciled",
        )
        prepared_identity = (
            log["publication"].get("prepared_directory_device"),
            log["publication"].get("prepared_directory_inode"),
        )
        require(
            all(isinstance(value, int) and not isinstance(value, bool) and value > 0 for value in prepared_identity),
            "publication intent lacks a prepared directory identity",
        )

        try:
            quarantine_stat = os.stat(
                quarantine_name, dir_fd=log_parent_fd, follow_symlinks=False
            )
        except FileNotFoundError:
            quarantine_stat = None
        try:
            final_stat = os.stat(
                final_name, dir_fd=final_parent_fd, follow_symlinks=False
            )
        except FileNotFoundError:
            final_stat = None
        require(
            (quarantine_stat is None) != (final_stat is None),
            "reconciliation requires exactly one prepared or published directory name",
        )
        if quarantine_stat is not None:
            require(
                stat.S_ISDIR(quarantine_stat.st_mode)
                and (quarantine_stat.st_dev, quarantine_stat.st_ino)
                == prepared_identity,
                "prepared quarantine identity changed",
            )
            batch_directory_fd = os.open(
                quarantine_name, DIRECTORY_FLAGS, dir_fd=log_parent_fd
            )
        else:
            require(
                final_stat is not None
                and stat.S_ISDIR(final_stat.st_mode)
                and (final_stat.st_dev, final_stat.st_ino) == prepared_identity,
                "published directory identity changed",
            )
            batch_directory_fd = os.open(
                final_name, DIRECTORY_FLAGS, dir_fd=final_parent_fd
            )
        opened_batch = os.fstat(batch_directory_fd)
        require(
            (opened_batch.st_dev, opened_batch.st_ino) == prepared_identity,
            "prepared directory changed during reconciliation open",
        )
        manifest_name = batch["filesystem_contract"]["sealed_manifest_name"]
        manifest_fd = _open_existing_regular(
            batch_directory_fd, manifest_name, writable=False
        )
        try:
            manifest_payload = _read_bounded_fd(
                manifest_fd, 2_097_152, "sealed manifest"
            )
        finally:
            os.close(manifest_fd)
        manifest_document = artifact_validator.parse_json(
            manifest_payload, "sealed manifest"
        )
        require(
            manifest_payload == artifacts.render(manifest_document),
            "sealed manifest bytes are not deterministic",
        )
        artifact_validator.validate_access_log_document(
            log,
            frozen_batch=frozen,
            repo_root=repo_root,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        artifact_validator.validate_sealed_directory_fd(
            batch_directory_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        _pass_checkpoint_guard = artifacts.build_implementation_snapshot(repo_root)
        require(
            _pass_checkpoint_guard["files"] == log["implementation_snapshot"]["files"]
            and _pass_checkpoint_guard["aggregate_sha256"]
            == log["implementation_snapshot"]["aggregate_sha256"],
            "implementation changed before reconciliation",
        )

        if quarantine_stat is not None:
            log_parent_chain.revalidate()
            final_parent_chain.revalidate()
            _assert_named_directory_identity(
                log_parent_fd,
                quarantine_name,
                batch_directory_fd,
                prepared_identity,
                "reconciliation quarantine",
            )
            _assert_name_absent(
                final_parent_fd, final_name, "final batch directory"
            )
            # As in the one-shot path, the sealed 0500 directory needs one
            # minimal 0700 window so a cross-parent directory rename may
            # update its `..` entry. Exact contents are checked on both sides.
            os.fchmod(batch_directory_fd, 0o700)
            try:
                acquisition.rename_directory_noreplace(
                    log_parent_fd,
                    quarantine_name,
                    final_parent_fd,
                    final_name,
                )
            finally:
                os.fchmod(batch_directory_fd, 0o500)
        os.fchmod(batch_directory_fd, 0o500)
        os.fsync(batch_directory_fd)
        os.fsync(log_parent_fd)
        os.fsync(final_parent_fd)
        published = os.stat(final_name, dir_fd=final_parent_fd, follow_symlinks=False)
        require(
            stat.S_ISDIR(published.st_mode)
            and (published.st_dev, published.st_ino) == prepared_identity,
            "reconciled final directory identity changed",
        )
        artifact_validator.validate_sealed_directory_fd(
            batch_directory_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        published_at = utc_now()
        for record in log["entries"]:
            record["state"] = "published"
        log["sealed_manifest"]["state"] = "published"
        log["publication"].update(
            {
                "state": "completed",
                "rename_count": 1,
                "source_parent_fsync_completed": True,
                "destination_parent_fsync_completed": True,
                "published_at_utc": published_at,
                "published_directory_device": published.st_dev,
                "published_directory_inode": published.st_ino,
            }
        )
        _transition(log, "completed", published_at)
        log["completed_at_utc"] = published_at
        artifact_validator.validate_access_log_document(
            log,
            frozen_batch=frozen,
            repo_root=repo_root,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        log_fd, terminal_log_payload = _persist_access_log_atomically(
            parent_fd=log_parent_fd,
            current_fd=log_fd,
            current_name=log_name,
            update_name=log_update_name,
            document=log,
        )
        log_parent_chain.revalidate()
        final_parent_chain.revalidate()
        _assert_named_regular_identity(
            log_parent_fd, log_name, log_fd, "reconciled completed access log"
        )
        _assert_named_directory_identity(
            final_parent_fd,
            final_name,
            batch_directory_fd,
            prepared_identity,
            "reconciled completed final batch",
        )
        artifact_validator.validate_sealed_directory_fd(
            batch_directory_fd,
            access_log=log,
            manifest_document=manifest_document,
            manifest_payload=manifest_payload,
        )
        terminal_hashes = artifact_validator.validate_repository_terminal(repo_root)
        require(
            terminal_hashes
            == (
                hashlib.sha256(terminal_log_payload).hexdigest(),
                hashlib.sha256(manifest_payload).hexdigest(),
            ),
            "reconciled terminal namespace hashes differ from durable artifacts",
        )
        return terminal_hashes
    finally:
        if batch_directory_fd is not None:
            os.close(batch_directory_fd)
        if log_fd is not None:
            os.close(log_fd)
        final_parent_chain.close()
        log_parent_chain.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--reconcile-publication-without-network",
        action="store_true",
        help="finish only an existing durable publication intent; never request bytes",
    )
    args = parser.parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    try:
        if args.reconcile_publication_without_network:
            log_hash, manifest_hash = reconcile_publication_without_network(repo_root)
        else:
            log_hash, manifest_hash = run_acquisition(repo_root)
    except (
        RunnerError,
        acquisition.AcquisitionError,
        artifact_validator.ContractError,
        batch_validator.ContractError,
        OSError,
        ssl.SSLError,
        socket.error,
        ValueError,
    ) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: RIOTBOX-1430 one-shot acquisition published exactly one sealed batch")
    print(f"acquisition_access_log_raw_sha256={log_hash}")
    print(f"acquisition_sealed_manifest_raw_sha256={manifest_hash}")
    print("source_feature_or_event_computation_performed=false")
    print("source_audio_playback_performed=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
