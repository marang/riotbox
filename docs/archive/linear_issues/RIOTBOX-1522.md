# `RIOTBOX-1522` Quote Feral grid verification command arguments without shell expansion

- Ticket: `RIOTBOX-1522`
- Title: `Quote Feral grid verification command arguments without shell expansion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1522/quote-feral-grid-verification-command-arguments-without-shell`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1522-literal-verification-argv`
- Linear branch: `feature/riotbox-1522-quote-feral-grid-verification-command-arguments-without`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`
- PR: `#1588 (https://github.com/marang/riotbox/pull/1588)`
- Merge commit: `d2d0a5d42ccc2e30c91945706e8af817a50ba63e`
- Deleted from Linear: `2026-10-01`
- Verification: `Original Debug and actual Release reds; final 46 unit plus one CLI integration in both profiles; 22 outer and 28 real local dry-run argv probes; 31 CLI exact results, 37/39 hashes exact including 16 WAVs/input/README/reports; two manifest changes only verification_command. Full source-free just ci and final warnings/errors scan green; native run 36808427065 Linux Rust and Windows Sidecar green, zero PR reviews/comments.`
- Docs touched: `docs/reviews/riotbox_1522_literal_verification_argv_2026-10-01.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1509 remains open; RIOTBOX-1523 next bounded timing-owner migration. Architecture cadence stays four.`

## Why This Ticket Existed

Literal source/date values were corrupted at both generated POSIX verification-command and Just recipe boundaries.

## What Shipped

- Private single-quote escaping for source/date, quote() at all twelve recipe interpolations; RBX-396 and owning shell grammar contract; adjacent regression and exact synthetic CLI tests.

## Notes

- Actual Just probe skipped on native runner as verified in log; static contract and Rust shell/CLI tests run. No real sources, holdout, commercial audio, devices, DAW/playback or human/music qualification. POSIX only, bounded QA recipe not exact Session replay.
