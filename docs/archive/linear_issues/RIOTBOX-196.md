# `RIOTBOX-196` Document the current source-backed W-30 limitation in the README quickstart

- Ticket: `RIOTBOX-196`
- Title: `Document the current source-backed W-30 limitation in the README quickstart`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-196/document-the-current-source-backed-w-30-limitation-in-the-readme`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-196-readme-w30-source-limit`
- Linear branch: `feature/riotbox-196-document-the-current-source-backed-w-30-limitation-in-the`
- PR: `#186`
- Merge commit: `26a9a69`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-197`

## Why This Ticket Existed

The README quickstart now points to the recipe guide, but the top-level user expectation still needed a concise note that W-30 source-backed playback is currently a bounded preview excerpt rather than a finished sampler engine.

## What Shipped

- Added a README quickstart note that W-30 capture reuse is intentionally bounded today.
- Pointed users to Recipe 11 for the current `.../src` versus `.../fallback` smoke path.
- Made explicit that Riotbox does not yet have a full W-30 sampler engine.

## Verification

- `git diff --check main..HEAD`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only slice; no runtime behavior changed.
- This keeps the public quickstart honest while the source-backed preview seam grows.
