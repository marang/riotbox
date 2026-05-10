# `RIOTBOX-251` Document queued timing rail hierarchy in TUI spec

- Ticket: `RIOTBOX-251`
- Title: `Document queued timing rail hierarchy in TUI spec`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-251/document-queued-timing-rail-hierarchy-in-tui-spec`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-251-document-queued-timing-rail-hierarchy-in-tui-spec`
- Linear branch: `feature/riotbox-251-document-queued-timing-rail-hierarchy-in-tui-spec`
- PR: `#241`
- Merge commit: `ad6f605`
- Labels: `ux`
- Follow-ups: `RIOTBOX-252`

## Why This Ticket Existed

`RIOTBOX-250` styled the Jam queued timing rail so countdown and boundary read as a performance cue rather than flat status text. The TUI spec needed to record that the rail remains monochrome-readable while visually emphasizing the snap point.

## What Shipped

- Documented queued timing rail hierarchy in `docs/specs/tui_screen_spec.md`.
- Specified yellow + bold emphasis for countdown glyphs and boundary labels.
- Kept beat, bar, and phrase counters as low-emphasis context.
- Clarified that the styling does not imply new scheduler behavior or a separate timing visualization widget.

## Verification

- `git diff --check main...HEAD`
- `rg -n 'queued timing rails|snap-point|scheduler behavior|timing visualization' docs/specs/tui_screen_spec.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Spec-only slice; no runtime behavior, screenshot baselines, new color semantics, or broad TUI spec rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
