#!/usr/bin/env python3
"""Deterministic source-only package; never read worktree sources or run the app."""

import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tomllib


LOCAL_PACKAGES = ("riotbox-app", "riotbox-audio", "riotbox-core", "riotbox-sidecar")
REQUIRED = (
    "Cargo.toml", "Cargo.lock", "LICENSE", "vendor/dawproject/Cargo.toml",
    "vendor/dawproject/PATCH.md", "vendor/dawproject/LICENSE-MIT",
    "vendor/dawproject/LICENSE-APACHE", "vendor/dawproject/upstream-source-sha256.json",
    *(f"crates/{name}/Cargo.toml" for name in LOCAL_PACKAGES),
)
TEXT_SUFFIXES = {
    ".c", ".jq", ".json", ".lock", ".map", ".md", ".ndjson", ".py", ".rs",
    ".sh", ".toml", ".txt", ".xsd", ".yaml", ".yml",
}
TEXT_NAMES = {
    "Justfile", "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", ".gitignore",
    ".rgignore", ".dockerignore",
}
FORBIDDEN_PARTS = {
    ".git", ".aws", ".ssh", "__pycache__", "target", "dist", "build", "artifacts",
    "private", "holdout", "holdouts", "commercial_references", "reference_audio",
    "source_audio", "sources", "sessions", "captures", "recordings", "review_packs",
    "credentials", "secrets", ".download-examples", ".mempalace",
}
PLACEHOLDER_ROOT = "crates/riotbox-audio/tests/fixtures/source_showcase_diversity"


class PackageError(ValueError):
    """An exact-commit source package failed its closed validation boundary."""


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def placeholder_bytes(path):
    """The 18 existing identity-test text files are not playable WAV recordings."""
    for beat in ("beat03", "beat08"):
        for case in ("valid", "invalid_full_mix", "invalid_dominated"):
            base = f"{PLACEHOLDER_ROOT}/{case}_{beat}"
            dominated = case == "invalid_dominated"
            expected = {
                f"{base}/03_riotbox_grid_feral_mix.wav": (
                    f"{beat} dominated full mix\n" if dominated else
                    "same full mix collapse\n" if case == "invalid_full_mix" else
                    f"{beat} full mix identity\n"
                ),
                f"{base}/stems/01_tr909_beat_fill.wav": (
                    "same dominant generated stem\n" if dominated else "same low support stem\n"
                ),
                f"{base}/stems/02_w30_feral_source_chop.wav": (
                    f"{beat} weak source chop\n" if dominated else f"{beat} source chop identity\n"
                ),
            }
            if path in expected:
                return expected[path].encode()
    return None


def safe_path(path):
    parts = path.split("/")
    if (not path or any(part in ("", ".", "..") for part in parts)
            or "\\" in path or ":" in path or any(ord(c) < 32 or ord(c) == 127 for c in path)):
        raise PackageError(f"unsafe package path: {path!r}")
    return PurePosixPath(path)


def source_path(path, size):
    name = safe_path(path)
    lowered = tuple(part.lower() for part in name.parts)
    # This existing production Rust module is code, not a private audio root.
    forbidden = FORBIDDEN_PARTS
    if path.startswith("crates/riotbox-audio/src/source_audio/") and name.suffix == ".rs":
        forbidden = forbidden - {"source_audio"}
    if (any(part in forbidden or part.startswith(".env") for part in lowered)
            or (name.suffix.lower() in (".json", ".ndjson") and re.search(
                r"(?:^|[_.-])(?:session|snapshot|capture|recording)(?:[_.-]|$)", name.name.lower()))):
        raise PackageError(f"private/generated path is not source distribution material: {path}")
    placeholder = placeholder_bytes(path)
    if placeholder is not None:
        if size != len(placeholder):
            raise PackageError(f"identity-test placeholder changed size: {path}")
    elif name.suffix == ".png" and path.startswith("docs/assets/"):
        pass
    elif name.suffix not in TEXT_SUFFIXES and name.name not in TEXT_NAMES:
        raise PackageError(f"non-source file type is not permitted: {path}")


def git(repo, *args, input_data=None):
    result = subprocess.run(
        ["git", "--no-replace-objects", "-c", "protocol.allow=never", "-C", str(repo), *args], input=input_data,
        capture_output=True, check=False,
    )
    if result.returncode:
        raise PackageError(f"git {args[0]} failed: {result.stderr.decode(errors='replace').strip()}")
    return result.stdout


def validate_sources(files, version):
    missing = sorted(set(REQUIRED) - files.keys())
    if missing:
        raise PackageError(f"required source/license files missing: {', '.join(missing)}")
    if any(not files[path][1].strip() for path in REQUIRED):
        raise PackageError("required source/license files must not be empty")
    try:
        workspace = tomllib.loads(files["Cargo.toml"][1].decode())
        lock = tomllib.loads(files["Cargo.lock"][1].decode())
        if workspace["workspace"]["package"]["version"] != version:
            raise PackageError("workspace version does not match requested release")
        for name in LOCAL_PACKAGES:
            matches = [p for p in lock["package"] if p["name"] == name]
            if len(matches) != 1 or matches[0]["version"] != version or "source" in matches[0]:
                raise PackageError(f"local lockfile version/source mismatch: {name}")
            crate = tomllib.loads(files[f"crates/{name}/Cargo.toml"][1].decode())
            inherited = crate["package"]["version"]
            if (crate["package"]["name"] != name or not isinstance(inherited, dict)
                    or set(inherited) != {"workspace"} or inherited["workspace"] is not True):
                raise PackageError(f"crate must inherit workspace version: {name}")
        upstream = json.loads(files["vendor/dawproject/upstream-source-sha256.json"][1])
        if not isinstance(upstream, dict) or not upstream:
            raise PackageError("vendor upstream hash record must not be empty")
        for path, digest in upstream.items():
            if not isinstance(path, str) or not isinstance(digest, str):
                raise PackageError("invalid vendor upstream hash record")
            safe_path(path)
            if not path.startswith("src/") or not re.fullmatch(r"[0-9a-f]{64}", digest):
                raise PackageError("invalid vendor upstream hash record")
            if f"vendor/dawproject/{path}" not in files:
                raise PackageError(f"vendor upstream source missing: {path}")
    except (KeyError, TypeError, UnicodeError, tomllib.TOMLDecodeError, json.JSONDecodeError) as exc:
        raise PackageError(f"invalid source manifests: {exc}") from exc


def committed_sources(repo, commit, version):
    if not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit):
        raise PackageError("--commit must be an exact full Git commit ID, not a moving ref")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", version):
        raise PackageError("invalid release version")
    resolved = git(repo, "rev-parse", "--verify", f"{commit}^{{commit}}").decode().strip()
    if resolved != commit:
        raise PackageError("requested object is not the exact commit")
    tree = git(repo, "rev-parse", f"{commit}^{{tree}}").decode().strip()
    entries = []
    for record in git(repo, "ls-tree", "-r", "-l", "-z", commit).split(b"\0"):
        if not record:
            continue
        header, raw_path = record.split(b"\t", 1)
        mode, kind, oid, raw_size = header.split()
        path = raw_path.decode("utf-8")
        if kind != b"blob" or mode not in (b"100644", b"100755"):
            raise PackageError(f"links/submodules/nonregular Git entry forbidden: {path}")
        size = int(raw_size)
        source_path(path, size)  # Reject forbidden paths before any blob reads.
        entries.append((path, int(mode, 8) & 0o777, oid, size))
    batch = io.BytesIO(git(repo, "cat-file", "--batch", input_data=b"".join(e[2] + b"\n" for e in entries)))
    files = {}
    for path, mode, oid, size in entries:
        if batch.readline().split() != [oid, b"blob", str(size).encode()]:
            raise PackageError(f"unexpected Git blob response: {path}")
        data = batch.read(size)
        if len(data) != size or batch.read(1) != b"\n":
            raise PackageError(f"incomplete Git blob: {path}")
        placeholder = placeholder_bytes(path)
        if placeholder is not None:
            if data != placeholder:
                raise PackageError(f"identity-test placeholder changed content: {path}")
        elif path.endswith(".png"):
            if not data.startswith(b"\x89PNG\r\n\x1a\n"):
                raise PackageError(f"invalid documentation image: {path}")
        else:
            if b"\0" in data or data.startswith((b"MZ", b"\x7fELF", b"RIFF", b"fLaC", b"OggS", b"ID3")):
                raise PackageError(f"binary/audio content is not source: {path}")
            data.decode("utf-8")
        if path in files:
            raise PackageError(f"duplicate Git tree path: {path}")
        files[path] = (mode, data)
    validate_sources(files, version)
    return tree, files


def package_bytes(files, version):
    prefix = f"riotbox-{version}/"
    output = io.BytesIO()
    with gzip.GzipFile(fileobj=output, mode="wb", filename="", mtime=0, compresslevel=9) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.PAX_FORMAT) as archive:
            for path, (mode, data) in sorted(files.items()):
                member = tarfile.TarInfo(prefix + path)
                member.size, member.mode, member.mtime = len(data), mode, 0
                archive.addfile(member, io.BytesIO(data))
    return output.getvalue()


def receipt(commit, tree, version, files, payload):
    return {
        "schema": "riotbox.source_package.v1", "commit": commit, "tree": tree, "version": version,
        "prefix": f"riotbox-{version}/",
        "archive": {"name": f"riotbox-{version}.tar.gz", "size_bytes": len(payload), "sha256": sha256(payload)},
        "files": [
            {"path": path, "mode": f"{mode:04o}", "size_bytes": len(data), "sha256": sha256(data)}
            for path, (mode, data) in sorted(files.items())
        ],
    }


def prepare(repo, commit, version, output):
    tree, files = committed_sources(repo, commit, version)
    payload = package_bytes(files, version)
    result = receipt(commit, tree, version, files, payload)
    output = Path(output)
    output.mkdir()  # New destination only, including rejection of existing empty directories/symlinks.
    with (output / result["archive"]["name"]).open("xb") as stream:
        stream.write(payload)
    with (output / "source-package.json").open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True)
        stream.write("\n")
    return result


def verify(repo, commit, version, archive, metadata=None):
    tree, files = committed_sources(repo, commit, version)
    expected = package_bytes(files, version)
    archive = Path(archive)
    if archive.stat().st_size != len(expected):
        raise PackageError("archive bytes differ from deterministic exact-commit source package")
    payload = archive.read_bytes()
    if len(payload) != len(expected) or sha256(payload) != sha256(expected):
        raise PackageError("archive bytes differ from deterministic exact-commit source package")
    # The digest gate bounds decompression and rejects corrupt trailers/trailing data first.
    prefix = f"riotbox-{version}/"
    seen = set()
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:gz") as packed:
        for member in packed:
            safe_path(member.name)
            if not member.isfile() or not member.name.startswith(prefix):
                raise PackageError(f"unsafe archive member type/prefix: {member.name}")
            path = member.name[len(prefix):]
            if path in seen or path not in files:
                raise PackageError(f"duplicate/extraneous archive member: {path}")
            seen.add(path)
            mode, data = files[path]
            if (member.mode != mode or member.size != len(data) or member.mtime != 0
                    or member.uid != 0 or member.gid != 0 or member.uname or member.gname
                    or packed.extractfile(member).read() != data):
                raise PackageError(f"archive member differs from commit: {path}")
    if seen != files.keys():
        raise PackageError("archive does not contain the complete source commit")
    result = receipt(commit, tree, version, files, expected)
    if metadata is not None and json.loads(Path(metadata).read_text()) != result:
        raise PackageError("package metadata does not match exact source/archive identity")
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for command in ("prepare", "verify"):
        child = commands.add_parser(command)
        child.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
        child.add_argument("--commit", required=True)
        child.add_argument("--version", required=True)
        if command == "prepare":
            child.add_argument("--output", type=Path, required=True)
        else:
            child.add_argument("--archive", type=Path, required=True)
            child.add_argument("--metadata", type=Path)
    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            result = prepare(args.repo, args.commit, args.version, args.output)
        else:
            result = verify(args.repo, args.commit, args.version, args.archive, args.metadata)
    except (PackageError, OSError, UnicodeError, json.JSONDecodeError) as exc:
        parser.exit(1, f"source package rejected: {exc}\n")
    print(json.dumps({key: value for key, value in result.items() if key != "files"}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
