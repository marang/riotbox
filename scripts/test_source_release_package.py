"""Generated fixture repos only: no product source audio, host IO, or Cargo."""

import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest import mock

import prepare_source_release as package


VERSION = "0.1.0-alpha.1"
SCRIPT = Path(package.__file__).resolve()


class SourcePackageTests(unittest.TestCase):
    def setUp(self):
        self.owner = tempfile.TemporaryDirectory(prefix="riotbox-package-test-")
        self.addCleanup(self.owner.cleanup)
        self.root = Path(self.owner.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.git("init", "--quiet")
        self.write("Cargo.toml", f'[workspace.package]\nversion = "{VERSION}"\n')
        self.write("Cargo.lock", "version = 4\n" + "".join(
            f'[[package]]\nname = "{name}"\nversion = "{VERSION}"\n'
            for name in package.LOCAL_PACKAGES
        ))
        for name in package.LOCAL_PACKAGES:
            self.write(f"crates/{name}/Cargo.toml", f'[package]\nname = "{name}"\nversion.workspace = true\n')
        for path in ("LICENSE", "vendor/dawproject/PATCH.md", "vendor/dawproject/LICENSE-MIT",
                     "vendor/dawproject/LICENSE-APACHE", "README.md"):
            self.write(path, f"Synthetic test-only {path}\n")
        self.write("vendor/dawproject/Cargo.toml", '[package]\nname = "dawproject"\nversion = "0.11.0"\n')
        self.write("vendor/dawproject/src/lib.rs", "// synthetic test library\n")
        self.write("vendor/dawproject/upstream-source-sha256.json", json.dumps({
            "src/lib.rs": hashlib.sha256(b"// synthetic test library\n").hexdigest(),
        }))
        self.write("scripts/check.sh", "#!/bin/sh\nexit 0\n")
        (self.repo / "scripts/check.sh").chmod(0o755)
        self.commit = self.commit_fixture()

    def git(self, *args):
        result = subprocess.run(
            ["git", "-C", str(self.repo), "-c", "user.name=Source Package Test",
             "-c", "user.email=source-package-test@example.invalid", "-c", "commit.gpgsign=false",
             "-c", "core.hooksPath=/dev/null", *args], capture_output=True, check=True,
        )
        return result.stdout.decode().strip()

    def write(self, path, value):
        destination = self.repo / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(value.encode() if isinstance(value, str) else value)

    def commit_fixture(self):
        self.git("add", "--all")
        self.git("commit", "--quiet", "-m", "synthetic source fixture")
        return self.git("rev-parse", "HEAD")

    def prepare(self, name="release", commit=None):
        output = self.root / name
        result = package.prepare(self.repo, commit or self.commit, VERSION, output)
        return output, result

    def archive(self, output):
        return output / f"riotbox-{VERSION}.tar.gz"

    def assert_rejected_commit(self, fragment):
        commit = self.commit_fixture()
        with self.assertRaisesRegex(package.PackageError, fragment):
            self.prepare(commit=commit)
        self.assertFalse((self.root / "release").exists())

    def assert_rejected_before_blobs(self, fragment):
        commit = self.commit_fixture()
        with mock.patch.object(package, "git", wraps=package.git) as observed:
            with self.assertRaisesRegex(package.PackageError, fragment):
                self.prepare(commit=commit)
        self.assertFalse(any("cat-file" in call.args for call in observed.call_args_list))

    def test_deterministic_new_directories_and_exact_identity_manifest(self):
        first, result = self.prepare("first")
        second, repeated = self.prepare("second")
        self.assertEqual(self.archive(first).read_bytes(), self.archive(second).read_bytes())
        self.assertEqual((first / "source-package.json").read_bytes(), (second / "source-package.json").read_bytes())
        self.assertEqual(result, repeated)
        self.assertEqual(result["commit"], self.commit)
        self.assertEqual(result["tree"], self.git("rev-parse", f"{self.commit}^{{tree}}"))
        self.assertEqual(result["archive"]["sha256"], hashlib.sha256(self.archive(first).read_bytes()).hexdigest())
        self.assertEqual(self.archive(first).read_bytes()[4:8], b"\0\0\0\0")
        self.assertEqual(package.verify(self.repo, self.commit, VERSION, self.archive(first), first / "source-package.json"), result)
        self.assertEqual(package.verify(self.repo, self.commit, VERSION, self.archive(first)), result)
        with tarfile.open(self.archive(first), "r:gz") as archive:
            self.assertEqual(archive.getnames(), [result["prefix"] + row["path"] for row in result["files"]])
            for row, member in zip(result["files"], archive.getmembers()):
                self.assertTrue(member.isfile())
                self.assertEqual((member.mtime, member.uid, member.gid, member.uname, member.gname), (0, 0, 0, "", ""))
                self.assertEqual(row["sha256"], hashlib.sha256(archive.extractfile(member).read()).hexdigest())
            self.assertEqual(archive.getmember(result["prefix"] + "scripts/check.sh").mode, 0o755)

    def test_dirty_tracked_and_untracked_files_never_enter_archive(self):
        self.write("Cargo.toml", "not TOML anymore\n")
        self.write("README.md", "DIRTY WORKTREE\n")
        self.write("sources/private.wav", b"RIFF generated forbidden fixture")
        output, result = self.prepare()
        paths = {row["path"] for row in result["files"]}
        self.assertNotIn("sources/private.wav", paths)
        with tarfile.open(self.archive(output), "r:gz") as archive:
            self.assertEqual(archive.extractfile(f"riotbox-{VERSION}/README.md").read(), b"Synthetic test-only README.md\n")
        package.verify(self.repo, self.commit, VERSION, self.archive(output))

    def test_real_source_audio_module_is_code_not_private_audio(self):
        path = "crates/riotbox-audio/src/source_audio/cache.rs"
        self.write(path, "// Representative production Rust module, not audio.\n")
        self.commit = self.commit_fixture()
        output, result = self.prepare()
        self.assertIn(path, {row["path"] for row in result["files"]})
        package.verify(self.repo, self.commit, VERSION, self.archive(output))
        # The exception is exact: neither a data root nor audio under code passes.
        for forbidden in ("source_audio/cache.rs",
                          "crates/riotbox-audio/src/source_audio/private.wav"):
            with self.subTest(path=forbidden):
                with self.assertRaises(package.PackageError):
                    package.source_path(forbidden, 12)

    def test_exact_old_commit_not_later_head(self):
        self.write("README.md", "later committed readme\n")
        newer = self.commit_fixture()
        output, _ = self.prepare()
        package.verify(self.repo, self.commit, VERSION, self.archive(output))
        with self.assertRaisesRegex(package.PackageError, "archive bytes differ"):
            package.verify(self.repo, newer, VERSION, self.archive(output))

    def test_existing_destination_and_symlink_are_never_overwritten(self):
        output, _ = self.prepare()
        before = self.archive(output).read_bytes()
        with self.assertRaises(FileExistsError):
            self.prepare()
        self.assertEqual(self.archive(output).read_bytes(), before)
        empty = self.root / "empty"
        empty.mkdir()
        with self.assertRaises(FileExistsError):
            self.prepare("empty")
        (self.root / "link").symlink_to(output, target_is_directory=True)
        with self.assertRaises(FileExistsError):
            self.prepare("link")

    def test_missing_required_license_and_vendor_records(self):
        for path in ("Cargo.lock", "LICENSE", "vendor/dawproject/LICENSE-MIT",
                     "vendor/dawproject/LICENSE-APACHE", "vendor/dawproject/PATCH.md",
                     "vendor/dawproject/upstream-source-sha256.json"):
            with self.subTest(path=path):
                existing = (self.repo / path).read_bytes()
                (self.repo / path).unlink()
                self.assert_rejected_commit("required source/license files missing")
                self.write(path, existing)

    def test_empty_license_rejected(self):
        self.write("LICENSE", "")
        self.assert_rejected_commit("must not be empty")

    def test_workspace_version_mismatch(self):
        self.write("Cargo.toml", '[workspace.package]\nversion = "0.1.0"\n')
        self.assert_rejected_commit("workspace version")

    def test_each_local_lock_version_must_match_once_and_be_local(self):
        original = (self.repo / "Cargo.lock").read_text()
        for name in package.LOCAL_PACKAGES:
            with self.subTest(name=name):
                self.write("Cargo.lock", original.replace(f'name = "{name}"\nversion = "{VERSION}"',
                                                          f'name = "{name}"\nversion = "0.1.0"'))
                self.assert_rejected_commit("local lockfile version/source mismatch")
        self.write("Cargo.lock", original + f'[[package]]\nname = "riotbox-app"\nversion = "{VERSION}"\n')
        self.assert_rejected_commit("local lockfile version/source mismatch")
        self.write("Cargo.lock", original + 'source = "registry+https://example.invalid"\n')
        self.assert_rejected_commit("local lockfile version/source mismatch")

    def test_crate_cannot_shadow_workspace_version(self):
        self.write("crates/riotbox-app/Cargo.toml", f'[package]\nname = "riotbox-app"\nversion = "{VERSION}"\n')
        self.assert_rejected_commit("must inherit workspace version")

    def test_only_exact_full_commit_and_safe_versions(self):
        for commit in ("HEAD", self.commit[:12], "--all", "v0.1.0-alpha.1"):
            with self.subTest(commit=commit), self.assertRaisesRegex(package.PackageError, "exact full Git commit"):
                package.prepare(self.repo, commit, VERSION, self.root / "release")
        for version in ("../../escape", "0.1.0/escape", "v0.1.0", ""):
            with self.subTest(version=version), self.assertRaisesRegex(package.PackageError, "invalid release version"):
                package.prepare(self.repo, self.commit, version, self.root / "release")

    def test_links_and_submodules_rejected_before_blob_access(self):
        (self.repo / "linked.rs").symlink_to("README.md")
        self.assert_rejected_before_blobs("links/submodules/nonregular")
        (self.repo / "linked.rs").unlink()
        self.git("add", "--all")
        self.git("update-index", "--add", "--cacheinfo", "160000", self.commit, "nested")
        self.git("commit", "--quiet", "-m", "synthetic submodule")
        with self.assertRaisesRegex(package.PackageError, "links/submodules/nonregular"):
            self.prepare(commit=self.git("rev-parse", "HEAD"))

    def test_private_audio_generated_output_and_credentials_rejected_before_blob_access(self):
        for path in ("sources/private.wav", "private/secret.json", "target/result.rs", "artifacts/review.json",
                     "sessions/take.json", "captures/take.json", "holdout/evidence.json", ".env.local",
                     ".aws/credentials", "samples/source.wav", "samples/source.flac", "bin/app.exe",
                     "session.json", "saved-session.json", "take.snapshot.json"):
            with self.subTest(path=path):
                self.write(path, "synthetic forbidden payload\n")
                self.assert_rejected_before_blobs("not source distribution material|non-source file type")
                (self.repo / path).unlink()

    def test_binary_or_audio_disguised_as_source_rejected(self):
        for data in (b"\x7fELFfake", b"MZfake", b"RIFFfake", b"text\0binary", b"\xffnot-utf8"):
            with self.subTest(data=data):
                self.write("extra.rs", data)
                commit = self.commit_fixture()
                with self.assertRaises((package.PackageError, UnicodeError)):
                    self.prepare(commit=commit)

    def test_all_eighteen_exact_text_wav_placeholders_are_allowed(self):
        paths = []
        for beat in ("beat03", "beat08"):
            for case in ("valid", "invalid_full_mix", "invalid_dominated"):
                for name in ("03_riotbox_grid_feral_mix.wav", "stems/01_tr909_beat_fill.wav", "stems/02_w30_feral_source_chop.wav"):
                    path = f"{package.PLACEHOLDER_ROOT}/{case}_{beat}/{name}"
                    self.write(path, package.placeholder_bytes(path))
                    paths.append(path)
        commit = self.commit_fixture()
        output, result = self.prepare(commit=commit)
        self.assertEqual(sum(row["path"].endswith(".wav") for row in result["files"]), 18)
        package.verify(self.repo, commit, VERSION, self.archive(output))
        self.write(paths[0], b"RIFF" + b"x" * (len(package.placeholder_bytes(paths[0])) - 4))
        with self.assertRaisesRegex(package.PackageError, "placeholder changed content"):
            self.prepare("bad-content", self.commit_fixture())
        self.write(paths[0], b"RIFFtoo long for the exact identity text fixture")
        with self.assertRaisesRegex(package.PackageError, "placeholder changed size"):
            self.prepare("bad-size", self.commit_fixture())

    def test_safe_paths_reject_cross_platform_traversal(self):
        for path in ("", "/absolute", "../escape", "ok/../../escape", "ok//file", "./file", "C:/file", "a\\b", "a\nfile", "x/./y"):
            with self.subTest(path=path), self.assertRaises(package.PackageError):
                package.safe_path(path)

    def test_malformed_archive_paths_links_duplicates_and_extra_audio_rejected(self):
        for name, kind in (("../escape", tarfile.REGTYPE), ("/absolute", tarfile.REGTYPE),
                           (f"riotbox-{VERSION}/README.md", tarfile.SYMTYPE),
                           (f"riotbox-{VERSION}/README.md", tarfile.LNKTYPE),
                           (f"riotbox-{VERSION}/extra.wav", tarfile.REGTYPE)):
            with self.subTest(name=name, kind=kind):
                raw = io.BytesIO()
                with tarfile.open(fileobj=raw, mode="w") as archive:
                    member = tarfile.TarInfo(name)
                    member.type = kind
                    member.linkname = "../../escape" if kind != tarfile.REGTYPE else ""
                    archive.addfile(member, io.BytesIO())
                    if kind == tarfile.REGTYPE:
                        archive.addfile(member, io.BytesIO())
                malformed = self.root / "malformed.tar.gz"
                malformed.write_bytes(gzip.compress(raw.getvalue(), mtime=0))
                with self.assertRaisesRegex(package.PackageError, "archive bytes differ"):
                    package.verify(self.repo, self.commit, VERSION, malformed)
                self.assertFalse((self.root / "escape").exists())

    def test_corruption_truncation_and_trailing_bytes_rejected(self):
        output, _ = self.prepare()
        original = self.archive(output).read_bytes()
        for altered in (original[:-1], original + b"trailing payload", original[:25] + bytes([original[25] ^ 1]) + original[26:]):
            self.archive(output).write_bytes(altered)
            with self.assertRaisesRegex(package.PackageError, "archive bytes differ"):
                package.verify(self.repo, self.commit, VERSION, self.archive(output))

    def test_metadata_cannot_be_rebound_or_omit_files(self):
        output, result = self.prepare()
        for key, value in (("commit", "0" * 40), ("tree", "0" * 40), ("version", "0.1.0"), ("files", [])):
            with self.subTest(key=key):
                edited = {**result, key: value}
                (output / "source-package.json").write_text(json.dumps(edited))
                with self.assertRaisesRegex(package.PackageError, "metadata does not match"):
                    package.verify(self.repo, self.commit, VERSION, self.archive(output), output / "source-package.json")

    def test_invalid_vendor_hash_record(self):
        for value in ({}, {"../outside.rs": "0" * 64}, {"src/lib.rs": "not-a-hash"},
                      {"src/missing.rs": "0" * 64}, {"src/lib.rs": None}):
            self.write("vendor/dawproject/upstream-source-sha256.json", json.dumps(value))
            self.assert_rejected_commit("vendor|unsafe package path")

    def test_cli_prepare_verify_and_clean_rejection(self):
        output = self.root / "cli"
        common = ["--repo", str(self.repo), "--commit", self.commit, "--version", VERSION]
        prepared = subprocess.run([sys.executable, str(SCRIPT), "prepare", *common, "--output", str(output)], capture_output=True, check=False)
        self.assertEqual(prepared.returncode, 0, prepared.stderr.decode())
        self.assertNotIn("files", json.loads(prepared.stdout))
        verified = subprocess.run([sys.executable, str(SCRIPT), "verify", *common, "--archive", str(self.archive(output)),
                                   "--metadata", str(output / "source-package.json")], capture_output=True, check=False)
        self.assertEqual(verified.returncode, 0, verified.stderr.decode())
        self.assertEqual(verified.stdout, prepared.stdout)
        rejected = subprocess.run([sys.executable, str(SCRIPT), "prepare", *common, "--output", str(output)], capture_output=True, check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn(b"source package rejected:", rejected.stderr)
        self.assertNotIn(b"Traceback", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
