# `RIOTBOX-1134` P016: Add stem-package restore and artifact-availability diagnostics

- Ticket: `RIOTBOX-1134`
- Title: `P016: Add stem-package restore and artifact-availability diagnostics`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1134/p016-add-stem-package-restore-and-artifact-availability-diagnostics`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1134-p016-add-stem-package-restore-artifact-availability-diagnostics`
- Linear branch: `feature/riotbox-1134-p016-add-stem-package-restore-and-artifact-availability`
- Assignee: `Markus`
- Labels: None
- PR: `#1112 (https://github.com/marang/riotbox/pull/1112)`
- Merge commit: `92fb9a10ac7f5696469aac50e18adf8d295cea42`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt`; `cargo test -p riotbox-app export_receipt_hydration_preflight -- --nocapture`; `cargo test -p riotbox-app recovery_surface_reports_ -- --nocapture`; `cargo test -p riotbox-app`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-1134-just-ci.log just ci`; `GitHub Actions rust-ci passed on PR #1112`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/replay_model_spec.md`
- Follow-ups: `RIOTBOX-1135 guarded execute CLI proof path; RIOTBOX-1136 operator proof summary command`

## Why This Ticket Existed

Stem-package restore needed read-only artifact diagnostics so recovery can tell ready packages, missing receipt identity, and missing local files apart without regenerating musician exports.

## What Shipped

- Export receipt artifact preflight now validates stem-package claimed roles from stem_package_artifact_set_evidence and requires each claimed stem WAV plus manifest/proof artifact-set identities.
- Recovery diagnostics distinguish blank or missing identity from unavailable local stem, manifest, or proof files while leaving product-mix receipt recovery unchanged.
- Added ready, missing stem WAV, missing manifest/proof identity/file, and unsupported legacy stem-package shape tests plus recovery-surface coverage.
- Session and Replay specs now state that stem-package recovery validation is read-only and must not rewrite, promote, or regenerate package artifacts.

## Notes

- human_verdict: unverified because this is read-only recovery diagnostics and does not change audible output.
