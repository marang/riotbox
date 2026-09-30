# `RIOTBOX-1337` P023: Split library CLI include shell into semantic modules

- Ticket: `RIOTBOX-1337`
- Title: `P023: Split library CLI include shell into semantic modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1337/p023-split-library-cli-include-shell-into-semantic-modules`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1337-semantic-cli`
- Linear branch: `feature/riotbox-1337-p023-split-library-cli-include-shell-into-semantic-modules`
- Assignee: `Markus`
- Labels: `Improvement`
- PR: `#1553 (https://github.com/marang/riotbox/pull/1553)`
- Merge commit: `99896a5c06a5a77b0f2629fe2fc1d9e12302a8c3`
- Deleted from Linear: `2026-09-30`
- Verification: `144 CLI tests green before/after, exact executable help bytes/channels/exit status unchanged, full local just ci App770 Audio279 Core470 Sidecar24, strict Clippy, include guard203sites16owners; native Ubuntu/Windows PR CI run36769376297 green.`
- Docs touched: `docs/reviews/riotbox_1337_semantic_cli_2026-09-30.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1411 owns the separate UI/Jam test shell migration and is In Progress.`

## Why This Ticket Existed

The thin app binary still delegated to a relocated 22-include CLI shell and 23-include regression namespace with hidden dependency ownership.

## What Shipped

- Real private configuration, arguments, launch, terminal, event, control, observer and offline-mode owners plus real regression modules replace all 45 includes without CLI behavior changes.

## Notes

- RBX-384; no new TUI features, public API, keys, observer schema, DSP, source or holdout access, DAW/device or human playback. Existing crate-internal W-30 handoff summary path preserved. Solo sequential review/self-review: zero unresolved findings, not independent approval.
