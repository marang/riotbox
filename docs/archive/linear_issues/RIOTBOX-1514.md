# `RIOTBOX-1514` Replace observer/audio QA correlation includes with semantic Rust owners

- Ticket: `RIOTBOX-1514`
- Title: `Replace observer/audio QA correlation includes with semantic Rust owners`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1514/replace-observeraudio-qa-correlation-includes-with-semantic-rust`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1514-semantic-observer-qa`
- Linear branch: `feature/riotbox-1514-replace-observeraudio-qa-correlation-includes-with-semantic`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1572 (https://github.com/marang/riotbox/pull/1572)`
- Merge commit: `348a13e88db9dd81e9d6e0af2be9e65463ec77ae`
- Deleted from Linear: `2026-10-01`
- Verification: `All 272 complete definitions retained: 268 normalized matches and four inspected Rustfmt expression-block differences; all 61 test leaves/statuses identical and pass; exact stdout/stderr/exit status in 21 metadata-only CLI cases; file-output JSON/Markdown bytes match baseline stdout; all-target/build/test logs explicitly warning-free; full source-free just ci and strict Clippy green; solo Rust/self-review no migration-introduced findings; native exact-head run 36795220369 both jobs green, no reviews/comments.`
- Docs touched: `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1514_semantic_observer_qa_2026-10-01.md`
- Follow-ups: `RIOTBOX-1515: demonstrated existing anchor-count u64 overflow (Debug panic; release wrapping code-derived); RIOTBOX-1516: mandatory architecture checkpoint now active before another structural migration.`

## Why This Ticket Existed

Expose semantic metadata-only QA parsing, evidence and presentation ownership without inventing another product or replay model.

## What Shipped

- Removed ten lexical includes; Cargo-standard main preserves the same single binary target; ordinary private report, metadata I/O, observer/manifest, alignment/evidence and JSON/Markdown owners plus explicit regression imports. Include inventory 54/7 to 44/6.

## Notes

- RBX-391 is ownership only. Fifth substantive architecture slice since RIOTBOX-1507; checkpoint reset only after its own merge/closeout. No schema, threshold, algorithm or sound change; no real source/holdout/reference/device/DAW/playback access. Existing validation-behavior defect remains a separate ready P2 bug, not hidden in the mechanical migration. No subagents.
