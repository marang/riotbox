#!/usr/bin/env python3
"""Synthetic no-network execution fixtures for the RIOTBOX-1430 runner."""

from __future__ import annotations

import errno
import hashlib
import json
import shutil
import struct
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any, Callable

import percussive_force_stage_a_v2_acquisition_artifacts as artifacts
import percussive_force_stage_a_v2_acquisition_contract as batch_contract
import run_percussive_force_stage_a_v2_acquisition as runner
import validate_percussive_force_stage_a_v2_acquisition_artifacts as artifact_validator


REPO = Path(__file__).resolve().parents[1]
CONTRACT_FILES = (
    "docs/benchmarks/percussive_force_stage_a_protocol_v1.json",
    "docs/benchmarks/percussive_force_stage_a_protocol_v2.json",
    "docs/benchmarks/percussive_force_development_matrix_v1.json",
    "docs/benchmarks/percussive_force_development_matrix_v2.json",
    "docs/benchmarks/source_holdout_rotation_v1.json",
    "docs/benchmarks/source_holdout_rotation_v2.json",
    "docs/benchmarks/percussive_force_stage_a_v2_acquisition_batch_v1.json",
)


def prepare_repo(destination: Path) -> None:
    for relative in (*CONTRACT_FILES, *artifacts.IMPLEMENTATION_FILES):
        source = REPO / relative
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    head = "1" * 40
    (destination / ".git/refs/heads").mkdir(parents=True)
    (destination / ".git/HEAD").write_text("ref: refs/heads/fixture\n")
    (destination / ".git/refs/heads/fixture").write_text(head + "\n")
    (destination / batch_contract.SOURCE_ROOT).mkdir(parents=True)


def wave_payload(byte_count: int, seed: int) -> bytes:
    data_size = byte_count - 44
    assert data_size > 0 and data_size % 2 == 0
    header = (
        b"RIFF"
        + struct.pack("<I", byte_count - 8)
        + b"WAVE"
        + b"fmt "
        + struct.pack("<IHHIIHH", 16, 1, 1, 192_000, 384_000, 2, 16)
        + b"data"
        + struct.pack("<I", data_size)
    )
    fill = bytes([seed]) * data_size
    return header + fill


class FakeResponse:
    def __init__(
        self,
        payload: bytes,
        *,
        status: int = 200,
        declared_length: int | None = None,
        content_type: str = "audio/x-wav",
    ) -> None:
        self.payload = payload
        self.offset = 0
        self.status = status
        self.declared_length = len(payload) if declared_length is None else declared_length
        self.content_type = content_type

    def getheaders(self) -> list[tuple[str, str]]:
        return [
            ("Content-Length", str(self.declared_length)),
            ("Content-Type", self.content_type),
        ]

    def read(self, count: int = -1) -> bytes:
        if count < 0:
            count = len(self.payload) - self.offset
        chunk = self.payload[self.offset : self.offset + count]
        self.offset += len(chunk)
        return chunk

    def close(self) -> None:
        return None


class FakeSession:
    def __init__(self, response: FakeResponse, on_close: Callable[[], None] | None = None) -> None:
        self.response = response
        self.connected_peer_ip = "1.1.1.1"
        self.negotiated_tls_version = "TLSv1.3"
        self.peer_certificate_sha256 = hashlib.sha256(b"fixture-certificate").hexdigest()
        self.on_close = on_close

    def close(self) -> None:
        self.response.close()
        if self.on_close is not None:
            self.on_close()


class NetworkFixture:
    def __init__(
        self,
        payload_mutator: Callable[[int, bytes], bytes] | None = None,
        status_for: Callable[[int], int] | None = None,
        on_close_for: Callable[[int], Callable[[], None] | None] | None = None,
    ) -> None:
        self.calls: list[int] = []
        self.dns_calls = 0
        self.payload_mutator = payload_mutator
        self.status_for = status_for
        self.on_close_for = on_close_for

    def resolve(self, _host: str) -> tuple[list[str], runner.ResolvedEndpoint]:
        self.dns_calls += 1
        return ["1.1.1.1"], runner.ResolvedEndpoint(
            runner.socket.AF_INET, ("1.1.1.1", 443), "1.1.1.1"
        )

    def open(self, _endpoint: runner.ResolvedEndpoint, request_target: str) -> FakeSession:
        batch = json.loads((REPO / batch_contract.BATCH_REL).read_text())
        from urllib.parse import urlsplit

        targets = [urlsplit(entry["download_url"]).path for entry in batch["entries"]]
        ordinal = targets.index(request_target) + 1
        self.calls.append(ordinal)
        byte_count = batch["entries"][ordinal - 1]["attachment_byte_count"]
        payload = wave_payload(byte_count, ordinal)
        if self.payload_mutator is not None:
            payload = self.payload_mutator(ordinal, payload)
        status = self.status_for(ordinal) if self.status_for is not None else 200
        on_close = self.on_close_for(ordinal) if self.on_close_for is not None else None
        return FakeSession(
            FakeResponse(payload, status=status, declared_length=byte_count),
            on_close=on_close,
        )


class PatchedNetwork:
    def __init__(self, fixture: NetworkFixture) -> None:
        self.fixture = fixture
        self.original_resolve = runner._resolve_one_public_endpoint
        self.original_open = runner._open_exact_https_response

    def __enter__(self) -> NetworkFixture:
        runner._resolve_one_public_endpoint = self.fixture.resolve
        runner._open_exact_https_response = self.fixture.open
        return self.fixture

    def __exit__(self, *_args: Any) -> None:
        runner._resolve_one_public_endpoint = self.original_resolve
        runner._open_exact_https_response = self.original_open


def exact_exists(repo: Path, relative: str) -> bool:
    return (repo / relative).exists()


def expect_failure(action: Callable[[], None]) -> None:
    try:
        action()
    except Exception:
        return
    raise AssertionError("runner fixture failed open")


def read_log(repo: Path) -> dict[str, Any]:
    path = repo / batch_contract.ACCESS_LOG_PATH
    return artifact_validator.parse_json(path.read_bytes(), str(path))


def run_success_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-runner-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original_rename = runner.acquisition.rename_directory_noreplace
        original_listdir = runner.os.listdir
        original_scandir = runner.os.scandir
        original_walk = runner.os.walk
        rename_calls: list[tuple[str, str]] = []

        def counted_rename(source_fd: int, source: str, destination_fd: int, destination: str) -> None:
            rename_calls.append((source, destination))
            original_rename(source_fd, source, destination_fd, destination)

        runner.acquisition.rename_directory_noreplace = counted_rename
        runner.os.listdir = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            AssertionError("directory listing is forbidden")
        )
        runner.os.scandir = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            AssertionError("directory scanning is forbidden")
        )
        runner.os.walk = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            AssertionError("directory walking is forbidden")
        )
        try:
            with PatchedNetwork(fixture):
                runner.run_acquisition(repo)
        finally:
            runner.acquisition.rename_directory_noreplace = original_rename
            runner.os.listdir = original_listdir
            runner.os.scandir = original_scandir
            runner.os.walk = original_walk
        assert fixture.calls == [1, 2, 3]
        assert fixture.dns_calls == 3
        assert rename_calls == [
            (
                runner.PUBLICATION_PROBE_SOURCE_NAME,
                runner.PUBLICATION_PROBE_DESTINATION_NAME,
            ),
            (
                PurePosixPath(batch_contract.QUARANTINE_DIRECTORY).name,
                PurePosixPath(batch_contract.FINAL_BATCH_DIRECTORY).name,
            ),
        ]
        assert exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        artifact_validator.validate_repository_terminal(repo)


def run_preexisting_fixture(relative: str, *, directory: bool) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-preexisting-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        target = repo / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if directory:
            target.mkdir()
            before = target.stat()
        else:
            target.write_text("already consumed\n")
            before_payload = target.read_bytes()
        fixture = NetworkFixture()
        with PatchedNetwork(fixture):
            expect_failure(lambda: runner.run_acquisition(repo))
        assert fixture.calls == []
        if directory:
            after = target.stat()
            assert (before.st_dev, before.st_ino) == (after.st_dev, after.st_ino)
        else:
            assert target.read_bytes() == before_payload


def run_failure_fixture(
    fixture: NetworkFixture,
    expected_calls: list[int],
) -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-failure-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        with PatchedNetwork(fixture):
            expect_failure(lambda: runner.run_acquisition(repo))
        assert fixture.calls == expected_calls
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        log = read_log(repo)
        assert log["attempt_status"] == "rejected"
        assert log["rejection"]["further_requests_performed"] is False


def run_reconciliation_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-reconcile-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        with PatchedNetwork(fixture):
            runner.run_acquisition(repo)
        assert fixture.calls == [1, 2, 3]
        log_path = repo / batch_contract.ACCESS_LOG_PATH
        log = read_log(repo)
        log["attempt_status"] = "publication_pending"
        log["completed_at_utc"] = None
        log["transition_history"].pop()
        for record in log["entries"]:
            record["state"] = "sealed"
        log["sealed_manifest"]["state"] = "sealed_in_quarantine"
        log["publication"].update(
            {
                "state": "pending",
                "rename_count": 0,
                "source_parent_fsync_completed": False,
                "destination_parent_fsync_completed": False,
                "published_at_utc": None,
                "published_directory_device": None,
                "published_directory_inode": None,
            }
        )
        log_path.write_bytes(artifacts.render(log))
        final_directory = repo / batch_contract.FINAL_BATCH_DIRECTORY
        quarantine_directory = repo / batch_contract.QUARANTINE_DIRECTORY
        final_directory.chmod(0o700)
        final_directory.rename(quarantine_directory)
        quarantine_directory.chmod(0o500)
        original_resolve = runner._resolve_one_public_endpoint
        original_open = runner._open_exact_https_response
        runner._resolve_one_public_endpoint = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            AssertionError("reconciliation attempted DNS")
        )
        runner._open_exact_https_response = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            AssertionError("reconciliation attempted network")
        )
        try:
            runner.reconcile_publication_without_network(repo)
        finally:
            runner._resolve_one_public_endpoint = original_resolve
            runner._open_exact_https_response = original_open
        terminal = read_log(repo)
        assert terminal["attempt_status"] == "completed"
        artifact_validator.validate_repository_terminal(repo)


def run_rename_failure_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-rename-failure-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner.acquisition.rename_directory_noreplace

        def reject_rename(
            source_fd: int,
            source: str,
            destination_fd: int,
            destination: str,
        ) -> None:
            if source == PurePosixPath(batch_contract.QUARANTINE_DIRECTORY).name:
                raise OSError(errno.EEXIST, "fixture destination exists")
            original(source_fd, source, destination_fd, destination)

        runner.acquisition.rename_directory_noreplace = reject_rename
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner.acquisition.rename_directory_noreplace = original
        assert fixture.calls == [1, 2, 3]
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        assert read_log(repo)["attempt_status"] == "rejected"


def run_publication_probe_failure_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-probe-failure-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner.acquisition.rename_directory_noreplace

        def reject_probe(*_args: Any) -> None:
            raise OSError(errno.ENOSYS, "fixture renameat2 unavailable")

        runner.acquisition.rename_directory_noreplace = reject_probe
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner.acquisition.rename_directory_noreplace = original
        assert fixture.calls == []
        assert not exact_exists(repo, batch_contract.ACCESS_LOG_PATH)
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(
            repo,
            str(
                PurePosixPath(batch_contract.QUARANTINE_DIRECTORY).parent
                / runner.PUBLICATION_PROBE_SOURCE_NAME
            ),
        )


def run_hash_collision_control_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-hash-collision-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner._assert_unique_payload_hash

        def reject_hash(*_args: Any, **_kwargs: Any) -> None:
            raise runner.RunnerError("payload_identity_collision_registry_v2")

        runner._assert_unique_payload_hash = reject_hash
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner._assert_unique_payload_hash = original
        assert fixture.calls == [1]
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert read_log(repo)["attempt_status"] == "rejected"


def prepare_post_rename_pending(repo: Path) -> NetworkFixture:
    fixture = NetworkFixture()
    original_rename = runner.acquisition.rename_directory_noreplace
    original_fchmod = runner.os.fchmod
    renamed = False

    def mark_rename(*args: Any) -> None:
        nonlocal renamed
        original_rename(*args)
        if args[1] == PurePosixPath(batch_contract.QUARANTINE_DIRECTORY).name:
            renamed = True

    def fail_after_rename(descriptor: int, mode: int) -> None:
        if renamed:
            raise OSError(errno.EIO, "fixture post-rename durability failure")
        original_fchmod(descriptor, mode)

    runner.acquisition.rename_directory_noreplace = mark_rename
    runner.os.fchmod = fail_after_rename
    try:
        with PatchedNetwork(fixture):
            expect_failure(lambda: runner.run_acquisition(repo))
    finally:
        runner.acquisition.rename_directory_noreplace = original_rename
        runner.os.fchmod = original_fchmod
    assert fixture.calls == [1, 2, 3]
    pending = read_log(repo)
    assert pending["attempt_status"] == "publication_pending"
    assert pending["publication"]["state"] == "pending"
    assert exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
    return fixture


def run_post_rename_recovery_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-post-rename-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        prepare_post_rename_pending(repo)
        runner.reconcile_publication_without_network(repo)
        artifact_validator.validate_repository_terminal(repo)


def run_implementation_drift_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-drift-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)

        def close_callback(ordinal: int) -> Callable[[], None] | None:
            if ordinal != 1:
                return None

            def drift() -> None:
                path = repo / "scripts/run_percussive_force_stage_a_v2_acquisition.py"
                path.write_bytes(path.read_bytes() + b"\n")

            return drift

        fixture = NetworkFixture(on_close_for=close_callback)
        with PatchedNetwork(fixture):
            expect_failure(lambda: runner.run_acquisition(repo))
        assert fixture.calls == [1]
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        assert exact_exists(repo, batch_contract.ACCESS_LOG_PATH)


def run_quarantine_namespace_swap_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-quarantine-swap-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)

        def close_callback(ordinal: int) -> Callable[[], None] | None:
            if ordinal != 1:
                return None

            def swap() -> None:
                quarantine = repo / batch_contract.QUARANTINE_DIRECTORY
                moved = quarantine.parent / "fixture-detached-quarantine"
                quarantine.rename(moved)
                quarantine.mkdir(mode=0o700)

            return swap

        fixture = NetworkFixture(on_close_for=close_callback)
        with PatchedNetwork(fixture):
            expect_failure(lambda: runner.run_acquisition(repo))
        assert fixture.calls == [1]
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        assert read_log(repo)["attempt_status"] == "rejected"


def run_atomic_log_write_failure_fixture(kind: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"riotbox-1430-log-{kind}-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        update_path = repo / (
            batch_contract.ACCESS_LOG_PATH + artifacts.ACCESS_LOG_UPDATE_SUFFIX
        )
        failed = False

        def is_update_descriptor(descriptor: int) -> bool:
            if not update_path.exists():
                return False
            named = update_path.stat()
            opened = runner.os.fstat(descriptor)
            return (named.st_dev, named.st_ino) == (opened.st_dev, opened.st_ino)

        original_pwrite_all = runner._pwrite_all
        original_ftruncate = runner.os.ftruncate
        original_fsync = runner.os.fsync

        def fail_pwrite(descriptor: int, payload: bytes) -> None:
            nonlocal failed
            if kind == "pwrite" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture pwrite failure")
            original_pwrite_all(descriptor, payload)

        def fail_ftruncate(descriptor: int, length: int) -> None:
            nonlocal failed
            if kind == "ftruncate" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture ftruncate failure")
            original_ftruncate(descriptor, length)

        def fail_fsync(descriptor: int) -> None:
            nonlocal failed
            if kind == "fsync" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture fsync failure")
            original_fsync(descriptor)

        runner._pwrite_all = fail_pwrite
        runner.os.ftruncate = fail_ftruncate
        runner.os.fsync = fail_fsync
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner._pwrite_all = original_pwrite_all
            runner.os.ftruncate = original_ftruncate
            runner.os.fsync = original_fsync
        assert failed
        assert fixture.calls == []
        assert not update_path.exists()
        log = read_log(repo)
        assert log["attempt_status"] == "rejected"


def run_reconcile_atomic_log_failure_fixture(kind: str) -> None:
    with tempfile.TemporaryDirectory(
        prefix=f"riotbox-1430-reconcile-log-{kind}-"
    ) as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        prepare_post_rename_pending(repo)
        update_path = repo / (
            batch_contract.ACCESS_LOG_PATH + artifacts.ACCESS_LOG_UPDATE_SUFFIX
        )
        failed = False
        successor_replaced = False

        def is_update_descriptor(descriptor: int) -> bool:
            if not update_path.exists():
                return False
            named = update_path.stat()
            opened = runner.os.fstat(descriptor)
            return (named.st_dev, named.st_ino) == (opened.st_dev, opened.st_ino)

        original_pwrite_all = runner._pwrite_all
        original_ftruncate = runner.os.ftruncate
        original_fsync = runner.os.fsync
        original_replace = runner.os.replace

        def fail_pwrite(descriptor: int, payload: bytes) -> None:
            nonlocal failed
            if kind == "pwrite" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture reconcile pwrite failure")
            original_pwrite_all(descriptor, payload)

        def fail_ftruncate(descriptor: int, length: int) -> None:
            nonlocal failed
            if kind == "ftruncate" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture reconcile ftruncate failure")
            original_ftruncate(descriptor, length)

        def fail_replace(*args: Any, **kwargs: Any) -> None:
            nonlocal failed, successor_replaced
            if kind == "replace" and not failed:
                failed = True
                raise OSError(errno.EIO, "fixture reconcile replace failure")
            original_replace(*args, **kwargs)
            successor_replaced = True

        def fail_fsync(descriptor: int) -> None:
            nonlocal failed
            if kind == "fsync" and not failed and is_update_descriptor(descriptor):
                failed = True
                raise OSError(errno.EIO, "fixture reconcile successor fsync failure")
            if kind == "parent_fsync" and successor_replaced and not failed:
                failed = True
                raise OSError(errno.EIO, "fixture reconcile parent fsync failure")
            original_fsync(descriptor)

        runner._pwrite_all = fail_pwrite
        runner.os.ftruncate = fail_ftruncate
        runner.os.fsync = fail_fsync
        runner.os.replace = fail_replace
        try:
            expect_failure(lambda: runner.reconcile_publication_without_network(repo))
        finally:
            runner._pwrite_all = original_pwrite_all
            runner.os.ftruncate = original_ftruncate
            runner.os.fsync = original_fsync
            runner.os.replace = original_replace
        assert failed
        assert not update_path.exists()
        if kind != "parent_fsync":
            assert read_log(repo)["attempt_status"] == "publication_pending"
            expect_failure(lambda: artifact_validator.validate_repository_terminal(repo))
        runner.reconcile_publication_without_network(repo)
        artifact_validator.validate_repository_terminal(repo)


def run_reconcile_torn_completed_successor_fixture() -> None:
    with tempfile.TemporaryDirectory(
        prefix="riotbox-1430-reconcile-log-torn-completed-"
    ) as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        prepare_post_rename_pending(repo)
        original = runner._persist_access_log_atomically

        def tear_completed(**kwargs: Any) -> tuple[int, bytes]:
            document = kwargs["document"]
            assert document["attempt_status"] == "completed"
            descriptor = runner._open_exclusive_regular(
                kwargs["parent_fd"], kwargs["update_name"], 0o600
            )
            try:
                runner.os.write(descriptor, b'{"torn"')
                runner.os.fsync(descriptor)
            finally:
                runner.os.close(descriptor)
            raise OSError(errno.EIO, "fixture reconcile torn completed successor")

        runner._persist_access_log_atomically = tear_completed
        try:
            expect_failure(lambda: runner.reconcile_publication_without_network(repo))
        finally:
            runner._persist_access_log_atomically = original
        assert read_log(repo)["attempt_status"] == "publication_pending"
        update_path = repo / (
            batch_contract.ACCESS_LOG_PATH + artifacts.ACCESS_LOG_UPDATE_SUFFIX
        )
        assert update_path.exists()
        runner.reconcile_publication_without_network(repo)
        assert not update_path.exists()
        artifact_validator.validate_repository_terminal(repo)


def run_terminal_namespace_swap_fixture(
    *, target_kind: str, reconcile: bool
) -> None:
    path_kind = "reconcile" if reconcile else "main"
    with tempfile.TemporaryDirectory(
        prefix=f"riotbox-1430-{path_kind}-{target_kind}-swap-"
    ) as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        if reconcile:
            prepare_post_rename_pending(repo)
        original = runner._persist_access_log_atomically
        swapped = False

        def swap_after_completed(**kwargs: Any) -> tuple[int, bytes]:
            nonlocal swapped
            descriptor, payload = original(**kwargs)
            if kwargs["document"]["attempt_status"] != "completed":
                return descriptor, payload
            if target_kind == "final_directory":
                target = repo / batch_contract.FINAL_BATCH_DIRECTORY
                detached = target.parent / "fixture-detached-final-directory"
                target.rename(detached)
                target.mkdir(mode=0o500)
            elif target_kind == "access_log":
                target = repo / batch_contract.ACCESS_LOG_PATH
                detached = target.parent / "fixture-detached-access-log"
                target.rename(detached)
                target.write_bytes(payload)
                target.chmod(0o600)
            else:
                raise AssertionError(f"unknown terminal swap target: {target_kind}")
            swapped = True
            return descriptor, payload

        runner._persist_access_log_atomically = swap_after_completed
        try:
            if reconcile:
                expect_failure(
                    lambda: runner.reconcile_publication_without_network(repo)
                )
            else:
                with PatchedNetwork(fixture):
                    expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner._persist_access_log_atomically = original
        assert swapped
        if not reconcile:
            assert fixture.calls == [1, 2, 3]
        if target_kind == "final_directory":
            expect_failure(
                lambda: artifact_validator.validate_repository_terminal(repo)
            )


def run_last_terminal_file_mutation_fixture(
    *, target_kind: str, mutation_kind: str
) -> None:
    with tempfile.TemporaryDirectory(
        prefix=f"riotbox-1430-last-terminal-{target_kind}-{mutation_kind}-"
    ) as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original_persist = runner._persist_access_log_atomically
        original_pread = artifact_validator.os.pread
        completed_committed = False
        target_eof_reads = 0
        mutated = False

        def mark_completed(**kwargs: Any) -> tuple[int, bytes]:
            nonlocal completed_committed
            result = original_persist(**kwargs)
            if kwargs["document"]["attempt_status"] == "completed":
                completed_committed = True
            return result

        def target_path_for(descriptor: int) -> Path | None:
            opened = artifact_validator.os.fstat(descriptor)
            final_directory = repo / batch_contract.FINAL_BATCH_DIRECTORY
            if target_kind == "payload":
                batch = json.loads(
                    (repo / batch_contract.BATCH_REL).read_text()
                )
                candidate = final_directory / PurePosixPath(
                    batch["entries"][0]["destination_path"]
                ).name
            elif target_kind == "manifest":
                candidate = final_directory / batch_contract.SEALED_MANIFEST_NAME
            elif target_kind == "access_log":
                candidate = repo / batch_contract.ACCESS_LOG_PATH
            else:
                raise AssertionError(f"unknown last-read target: {target_kind}")
            try:
                named = candidate.stat()
            except FileNotFoundError:
                return None
            if (named.st_dev, named.st_ino) != (opened.st_dev, opened.st_ino):
                return None
            return candidate

        def mutate_on_last_eof(
            descriptor: int, count: int, offset: int
        ) -> bytes:
            nonlocal target_eof_reads, mutated
            payload = original_pread(descriptor, count, offset)
            if not completed_committed or count != 1 or payload != b"":
                return payload
            target = target_path_for(descriptor)
            if target is None or offset != artifact_validator.os.fstat(descriptor).st_size:
                return payload
            target_eof_reads += 1
            final_read_ordinal = {
                "access_log": 1,
                "payload": 2,
                "manifest": 3,
            }[target_kind]
            if target_eof_reads != final_read_ordinal:
                return payload
            original_payload = target.read_bytes()
            changed_payload = bytes([original_payload[0] ^ 1]) + original_payload[1:]
            if mutation_kind == "name_swap":
                final_directory = target.parent
                detached = final_directory / f"fixture-detached-{target.name}"
                final_directory.chmod(0o700)
                target.rename(detached)
                target.write_bytes(changed_payload)
                target.chmod(0o400)
                final_directory.chmod(0o500)
            elif mutation_kind == "in_place":
                target.chmod(0o600)
                target.write_bytes(changed_payload)
                target.chmod(0o600 if target_kind == "access_log" else 0o400)
            else:
                raise AssertionError(
                    f"unknown last-read mutation kind: {mutation_kind}"
                )
            mutated = True
            return payload

        runner._persist_access_log_atomically = mark_completed
        artifact_validator.os.pread = mutate_on_last_eof
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            artifact_validator.os.pread = original_pread
            runner._persist_access_log_atomically = original_persist
        assert completed_committed
        assert mutated
        assert fixture.calls == [1, 2, 3]
        expect_failure(lambda: artifact_validator.validate_repository_terminal(repo))


def run_torn_completed_successor_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-log-torn-completed-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner._persist_access_log_atomically

        def tear_completed(**kwargs: Any) -> tuple[int, bytes]:
            document = kwargs["document"]
            if document["attempt_status"] != "completed":
                return original(**kwargs)
            descriptor = runner._open_exclusive_regular(
                kwargs["parent_fd"], kwargs["update_name"], 0o600
            )
            try:
                runner.os.write(descriptor, b'{"torn"')
                runner.os.fsync(descriptor)
            finally:
                runner.os.close(descriptor)
            raise OSError(errno.EIO, "fixture crash during completed successor")

        runner._persist_access_log_atomically = tear_completed
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner._persist_access_log_atomically = original
        pending = read_log(repo)
        assert pending["attempt_status"] == "publication_pending"
        update_path = repo / (
            batch_contract.ACCESS_LOG_PATH + artifacts.ACCESS_LOG_UPDATE_SUFFIX
        )
        assert update_path.exists()
        runner.reconcile_publication_without_network(repo)
        assert not update_path.exists()
        artifact_validator.validate_repository_terminal(repo)


def run_dns_fixtures() -> None:
    original = runner.socket.getaddrinfo
    calls = 0

    def mixed(host: str, *_args: Any, **_kwargs: Any) -> list[tuple[Any, ...]]:
        nonlocal calls
        calls += 1
        assert host == "opengameart.org."
        return [
            (runner.socket.AF_INET, runner.socket.SOCK_STREAM, runner.socket.IPPROTO_TCP, "", ("1.1.1.1", 443)),
            (runner.socket.AF_INET, runner.socket.SOCK_STREAM, runner.socket.IPPROTO_TCP, "", ("127.0.0.1", 443)),
        ]

    runner.socket.getaddrinfo = mixed
    try:
        expect_failure(lambda: runner._resolve_one_public_endpoint("opengameart.org"))
    finally:
        runner.socket.getaddrinfo = original
    assert calls == 1


def run_low_space_preflight_fixture() -> None:
    with tempfile.TemporaryDirectory(prefix="riotbox-1430-low-space-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner.os.fstatvfs

        class LowSpace:
            f_bavail = 0
            f_frsize = 4096

        runner.os.fstatvfs = lambda _descriptor: LowSpace()
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner.os.fstatvfs = original
        assert fixture.calls == []
        assert not exact_exists(repo, batch_contract.ACCESS_LOG_PATH)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)


def run_prepublication_sealed_mutation_fixture(target_kind: str) -> None:
    with tempfile.TemporaryDirectory(
        prefix=f"riotbox-1430-prepublication-{target_kind}-"
    ) as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        original = runner._pass_checkpoint
        mutated = False

        def mutate_after_checkpoint(
            log: dict[str, Any],
            checkpoint_index: int,
            **kwargs: Any,
        ) -> None:
            nonlocal mutated
            original(log, checkpoint_index, **kwargs)
            if checkpoint_index != 3:
                return
            quarantine = repo / batch_contract.QUARANTINE_DIRECTORY
            if target_kind == "payload":
                target = repo / log["entries"][0]["destination_path"]
                target = quarantine / target.name
            elif target_kind == "manifest":
                target = quarantine / batch_contract.SEALED_MANIFEST_NAME
            else:
                raise AssertionError(f"unknown sealed mutation target: {target_kind}")
            target.chmod(0o600)
            payload = target.read_bytes()
            target.write_bytes(bytes([payload[0] ^ 1]) + payload[1:])
            target.chmod(0o400)
            mutated = True

        runner._pass_checkpoint = mutate_after_checkpoint
        try:
            with PatchedNetwork(fixture):
                expect_failure(lambda: runner.run_acquisition(repo))
        finally:
            runner._pass_checkpoint = original
        assert mutated
        assert fixture.calls == [1, 2, 3]
        assert not exact_exists(repo, batch_contract.FINAL_BATCH_DIRECTORY)
        assert not exact_exists(repo, batch_contract.QUARANTINE_DIRECTORY)
        assert read_log(repo)["attempt_status"] == "rejected"


def run_exact_transport_fixtures() -> None:
    original_socket = runner.socket.socket
    original_context = runner.ssl.create_default_context
    original_response = runner.http.client.HTTPResponse

    class RawSocket:
        def __init__(self, peer: str) -> None:
            self.peer = peer
            self.connect_count = 0
            self.closed = False

        def settimeout(self, _timeout: float) -> None:
            return None

        def connect(self, _sockaddr: tuple[Any, ...]) -> None:
            self.connect_count += 1

        def getpeername(self) -> tuple[str, int]:
            return self.peer, 443

        def close(self) -> None:
            self.closed = True

    class TlsSocket(RawSocket):
        def __init__(self, raw: RawSocket, peer: str) -> None:
            super().__init__(peer)
            self.raw = raw
            self.sent = b""

        def version(self) -> str:
            return "TLSv1.3"

        def getpeercert(self, *, binary_form: bool = False) -> bytes:
            assert binary_form
            return b"certificate"

        def sendall(self, payload: bytes) -> None:
            self.sent += payload

    class Context:
        def __init__(self, tls_peer: str) -> None:
            self.minimum_version = None
            self.check_hostname = None
            self.verify_mode = None
            self.tls_peer = tls_peer
            self.sni = None
            self.last_tls: TlsSocket | None = None

        def wrap_socket(self, raw: RawSocket, *, server_hostname: str) -> TlsSocket:
            self.sni = server_hostname
            self.last_tls = TlsSocket(raw, self.tls_peer)
            return self.last_tls

    class Response:
        def __init__(self, tls: TlsSocket, *, method: str) -> None:
            assert method == "GET"
            self.tls = tls

        def begin(self) -> None:
            return None

        def close(self) -> None:
            return None

    raw_instances: list[RawSocket] = []
    context = Context("1.1.1.1")

    def make_socket(*_args: Any, **_kwargs: Any) -> RawSocket:
        raw = RawSocket("1.1.1.1")
        raw_instances.append(raw)
        return raw

    runner.socket.socket = make_socket
    runner.ssl.create_default_context = lambda: context
    runner.http.client.HTTPResponse = Response
    try:
        session = runner._open_exact_https_response(
            runner.ResolvedEndpoint(runner.socket.AF_INET, ("1.1.1.1", 443), "1.1.1.1"),
            "/sites/default/files/exact.wav",
        )
        session.close()
    finally:
        runner.socket.socket = original_socket
        runner.ssl.create_default_context = original_context
        runner.http.client.HTTPResponse = original_response
    assert len(raw_instances) == 1 and raw_instances[0].connect_count == 1
    assert context.sni == "opengameart.org"
    expected_request = (
        "GET /sites/default/files/exact.wav HTTP/1.1\r\n"
        + "\r\n".join(f"{name}: {value}" for name, value in artifacts.REQUEST_HEADERS)
        + "\r\n\r\n"
    ).encode("ascii")
    assert context.last_tls is not None and context.last_tls.sent == expected_request

    mismatch_instances: list[RawSocket] = []

    def mismatch_socket(*_args: Any, **_kwargs: Any) -> RawSocket:
        raw = RawSocket("8.8.8.8")
        mismatch_instances.append(raw)
        return raw

    runner.socket.socket = mismatch_socket
    try:
        expect_failure(
            lambda: runner._open_exact_https_response(
                runner.ResolvedEndpoint(runner.socket.AF_INET, ("1.1.1.1", 443), "1.1.1.1"),
                "/sites/default/files/exact.wav",
            )
        )
    finally:
        runner.socket.socket = original_socket
    assert len(mismatch_instances) == 1 and mismatch_instances[0].connect_count == 1


def run_terminal_integrity_failure_fixture(mode: str) -> None:
    with tempfile.TemporaryDirectory(prefix=f"riotbox-1430-terminal-{mode}-") as temporary:
        repo = Path(temporary)
        prepare_repo(repo)
        fixture = NetworkFixture()
        with PatchedNetwork(fixture):
            runner.run_acquisition(repo)
        batch = json.loads((repo / batch_contract.BATCH_REL).read_text())
        first = repo / batch["entries"][0]["destination_path"]
        final_directory = repo / batch_contract.FINAL_BATCH_DIRECTORY
        if mode == "missing":
            final_directory.chmod(0o700)
            first.unlink()
        elif mode == "tamper":
            first.chmod(0o600)
            with first.open("r+b") as target:
                target.seek(64)
                original = target.read(1)
                target.seek(64)
                target.write(bytes([original[0] ^ 1]))
                target.flush()
                runner.os.fsync(target.fileno())
        elif mode == "directory_identity":
            parent = final_directory.parent
            moved = parent / "fixture-old-directory"
            final_directory.rename(moved)
            moved.chmod(0o700)
            final_directory.mkdir(mode=0o700)
            names = [
                PurePosixPath(entry["destination_path"]).name
                for entry in batch["entries"]
            ] + [batch_contract.SEALED_MANIFEST_NAME]
            for name in names:
                (moved / name).rename(final_directory / name)
            moved.rmdir()
        else:
            raise AssertionError(f"unknown terminal fixture mode: {mode}")
        expect_failure(lambda: artifact_validator.validate_repository_terminal(repo))


def main() -> int:
    run_success_fixture()
    run_preexisting_fixture(batch_contract.ACCESS_LOG_PATH, directory=False)
    run_preexisting_fixture(
        batch_contract.ACCESS_LOG_PATH + artifacts.ACCESS_LOG_UPDATE_SUFFIX,
        directory=False,
    )
    run_preexisting_fixture(batch_contract.QUARANTINE_DIRECTORY, directory=True)
    run_preexisting_fixture(batch_contract.FINAL_BATCH_DIRECTORY, directory=True)
    run_failure_fixture(
        NetworkFixture(status_for=lambda ordinal: 302 if ordinal == 1 else 200),
        [1],
    )
    run_failure_fixture(
        NetworkFixture(
            payload_mutator=lambda ordinal, payload: payload[:-1]
            if ordinal == 1
            else payload
        ),
        [1],
    )
    run_failure_fixture(
        NetworkFixture(
            payload_mutator=lambda ordinal, payload: payload + b"x"
            if ordinal == 1
            else payload
        ),
        [1],
    )
    run_failure_fixture(
        NetworkFixture(
            payload_mutator=lambda ordinal, payload: (
                payload[:20] + b"\x03\x00" + payload[22:]
                if ordinal == 2
                else payload
            )
        ),
        [1, 2],
    )
    run_reconciliation_fixture()
    run_rename_failure_fixture()
    run_publication_probe_failure_fixture()
    run_hash_collision_control_fixture()
    run_post_rename_recovery_fixture()
    run_implementation_drift_fixture()
    run_quarantine_namespace_swap_fixture()
    run_atomic_log_write_failure_fixture("pwrite")
    run_atomic_log_write_failure_fixture("ftruncate")
    run_atomic_log_write_failure_fixture("fsync")
    run_reconcile_atomic_log_failure_fixture("pwrite")
    run_reconcile_atomic_log_failure_fixture("ftruncate")
    run_reconcile_atomic_log_failure_fixture("fsync")
    run_reconcile_atomic_log_failure_fixture("replace")
    run_reconcile_atomic_log_failure_fixture("parent_fsync")
    run_reconcile_torn_completed_successor_fixture()
    run_terminal_namespace_swap_fixture(
        target_kind="final_directory", reconcile=False
    )
    run_terminal_namespace_swap_fixture(target_kind="access_log", reconcile=False)
    run_terminal_namespace_swap_fixture(
        target_kind="final_directory", reconcile=True
    )
    run_terminal_namespace_swap_fixture(target_kind="access_log", reconcile=True)
    run_last_terminal_file_mutation_fixture(
        target_kind="payload", mutation_kind="name_swap"
    )
    run_last_terminal_file_mutation_fixture(
        target_kind="manifest", mutation_kind="name_swap"
    )
    run_last_terminal_file_mutation_fixture(
        target_kind="access_log", mutation_kind="in_place"
    )
    run_last_terminal_file_mutation_fixture(
        target_kind="payload", mutation_kind="in_place"
    )
    run_last_terminal_file_mutation_fixture(
        target_kind="manifest", mutation_kind="in_place"
    )
    run_torn_completed_successor_fixture()
    run_dns_fixtures()
    run_low_space_preflight_fixture()
    run_prepublication_sealed_mutation_fixture("payload")
    run_prepublication_sealed_mutation_fixture("manifest")
    run_exact_transport_fixtures()
    run_terminal_integrity_failure_fixture("missing")
    run_terminal_integrity_failure_fixture("tamper")
    run_terminal_integrity_failure_fixture("directory_identity")
    print("PASS: 43 no-network one-shot runner fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
