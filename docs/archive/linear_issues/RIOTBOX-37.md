# `RIOTBOX-37` Ticket Archive

- Ticket: `RIOTBOX-37`
- Title: `Resolve single-source app behavior versus plural session graph contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-37/resolve-single-source-app-behavior-versus-plural-session-graph`
- Project: `P003 | Analysis Vertical Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-13`
- Finished: `2026-04-15`
- Branch: `riotbox-37-single-source-session-contract`
- Assignee: `Markus`
- Labels: `Docs`, `TUI`, `Core`
- PR: `#30`
- Merge commit: `b823eaa`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The periodic review found that the app behaved as single-source while the core session shape still implied plural source and graph references, leaving the real MVP contract implicit.

## What Shipped

- Made the MVP single-source assumption explicit in app behavior.
- Documented the single-source constraint in the session-file spec.
- Recorded the decision so future multi-source work becomes an intentional design step instead of an accidental mismatch.

## Notes

- This intentionally chose a narrower, honest MVP contract over pretending to support multi-source sessions prematurely.
