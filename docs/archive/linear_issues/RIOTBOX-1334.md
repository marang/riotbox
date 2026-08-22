# `RIOTBOX-1334` P023: Harden sidecar protocol versioning and provenance boundaries

- Ticket: `RIOTBOX-1334`
- Title: `P023: Harden sidecar protocol versioning and provenance boundaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1334/p023-harden-sidecar-protocol-versioning-and-provenance-boundaries`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-08-22`
- Finished: `2026-08-22`
- Branch: `feature/riotbox-1334-p023-harden-sidecar-protocol-versioning-and-provenance`
- Linear branch: `feature/riotbox-1334-p023-harden-sidecar-protocol-versioning-and-provenance`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `review-followup`
- PR: `#1432 (https://github.com/marang/riotbox/pull/1432)`
- Merge commit: `c876ee89313cb23b7ad5666b2096a001c0622677`
- Deleted from Linear: `2026-08-22`
- Verification: `Final full just ci passed locally after review fixes.`; `GitHub rust-ci passed on PR #1432.`; `Focused Rust/Python sidecar tests, outside-CWD integration, formatting, Clippy -D warnings, and code/Rust review passed with no remaining findings.`
- Docs touched: `docs/README.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, `docs/plans/riotbox_improvement_tracks_plan.md`, `docs/research_decision_log.md`, `docs/specs/source_graph_spec.md`, `docs/specs/technology_stack_spec.md`
- Follow-ups: `Select the next highest-value unblocked audible Golden Path slice Linear-first; this contract enabler does not choose or authorize a mechanism.`

## Why This Ticket Existed

Close the final named Foundation trust gap so incompatible, stale, masked, or CWD-dependent sidecar state cannot be mistaken for a trustworthy Source Graph.

## What Shipped

- Validated protocol compatibility before graph acceptance and preserved request-less provider errors as their real typed failures.
- Added truthful injected-clock provenance, configurable bounded control/analysis deadlines, and CWD-independent bundled-sidecar discovery with explicit overrides.
- Rejected empty or stub provider sets on normal source-file analysis and added static golden protocol plus outside-CWD regressions.

## Notes

- No Development/Holdout audio, source directory, commercial reference, or human playback was accessed.
