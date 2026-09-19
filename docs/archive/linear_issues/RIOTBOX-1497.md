# `RIOTBOX-1497` Define and enforce persisted capture-WAV content identity on restore

- Ticket: `RIOTBOX-1497`
- Title: `Define and enforce persisted capture-WAV content identity on restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1497/define-and-enforce-persisted-capture-wav-content-identity-on-restore`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-19`
- Started: `2026-09-19`
- Finished: `2026-09-19`
- Branch: `feature/riotbox-1497-capture-identity`
- Linear branch: `feature/riotbox-1497-define-and-enforce-persisted-capture-wav-content-identity-on`
- Assignee: `Markus`
- Labels: `Core`, `review-followup`
- PR: `#1523 (https://github.com/marang/riotbox/pull/1523)`
- Merge commit: `0cd338b3aa7731c910d19b4269e018d46bdcb7c0`
- Deleted from Linear: `2026-09-19`
- Verification: `Local source-free just ci passed; 749 App tests plus workspace/CLI/synthetic audio/contracts/Clippy. GitHub Rust CI 35464315105 passed.`
- Docs touched: `docs/reviews/riotbox_1497_capture_identity_2026-09-19.md`
- Follow-ups: `None`

## Why This Ticket Existed

Capture restore accepted decodable replacements without a persisted content identity; old Sessions needed an honest explicit migration.

## What Shipped

- Core CaptureRef full-WAV SHA-256 with typed created/adopted provenance; same-buffer restore verification and fail-closed cache/preview/resample paths.
- Selected legacy capture migration: preview default, explicit current-content acceptance, atomic metadata update, no replacement of existing identities.
- Honest path-only recovery label, runtime warnings, RBX-375 and Session/replay contracts.

## Notes

- Solo review; no subagents, real Session migration, real source/holdout access or human listening. Existing single-writer save limits apply; adopted hashes do not establish historical authenticity.
