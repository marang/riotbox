# `RIOTBOX-255` Document latest landed result hierarchy in TUI spec

- Ticket: `RIOTBOX-255`
- Title: `Document latest landed result hierarchy in TUI spec`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-255/document-latest-landed-result-hierarchy-in-tui-spec`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-255-document-latest-landed-result-hierarchy-in-tui-spec`
- Linear branch: `feature/riotbox-255-document-latest-landed-result-hierarchy-in-tui-spec`
- PR: `#245`
- Merge commit: `4e825c5`
- Labels: `ux`
- Follow-ups: `RIOTBOX-256`

## Why This Ticket Existed

`RIOTBOX-254` styled the `landed ...` line in the Jam `Next` stack so the immediate committed result reads as confirmation instead of flat status text. The TUI spec needed to record that landed results are part of the same queued -> landed -> next hierarchy.

## What Shipped

- Documented latest-landed result cues in `docs/specs/tui_screen_spec.md`.
- Added emphasis guidance for committed commands, Scene energy direction, and low-emphasis actor/separator context.
- Preserved the plain-text sentence contract.

## Verification

- `git diff --check main...HEAD`
- `rg -n 'latest-landed|actually landed|landed user scene jump|actor labels' docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Spec-only slice; no runtime behavior, Log redesign, screenshot baselines, new color semantics, or broad TUI spec rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
