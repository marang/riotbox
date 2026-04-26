# Periodic Jam Hierarchy Seam Review 2026-04-26

Scope:

- focused current-state review of the Jam / TUI hierarchy seam after `RIOTBOX-248` through `RIOTBOX-255`
- reviewed `crates/riotbox-app/src/ui.rs` and the Jam hierarchy / emphasis-token contract in `docs/specs/tui_screen_spec.md`
- review type is current-state architecture/design review via `review-codebase`, not a diff-only review

## Findings

### 1. Primary `Next` stack still places landed history before the timing rail

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:662), [docs/specs/tui_screen_spec.md](/home/markus/Dev/riotbox/docs/specs/tui_screen_spec.md:195)
- Category: `scope`
- Severity: `minor`

The `Next` panel renders `next_action_line`, pending Scene intent, latest landed history, then the queued timing rail. That keeps the text readable, but it weakens the performance hierarchy: the timing rail is the imminent snap point for a queued action, while `latest_landed_line` is recent history. The spec ranks imminent committed change above history and diagnostics, so this order still makes the player read through the past before the musical boundary.

Suggestion:

- move the queued timing rail above latest landed history in the primary Jam `Next` panel
- keep the existing landed line visible below the timing rail as confirmation/history
- keep the existing text contract and update focused shell/style tests for order, not scheduler behavior

Follow-up: `RIOTBOX-257`

### 2. Semantic TUI styles are repeated as raw color/modifier literals

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1331), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1743), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1847), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3184), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3278), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3562)
- Category: `scope`
- Severity: `minor`

The current hierarchy uses the same small semantic palette consistently today, but the actual styles are encoded directly in each renderer: cyan + bold for primary controls, yellow + bold for pending/snap cues, green + bold for confirmations, and dark gray for context. That is acceptable for the first slices, but it creates a drift point now that footer, Capture, Scene post-commit, pending Scene intent, timing rail, and landed-result helpers all participate in the same contract.

Suggestion:

- add a small local helper layer for semantic TUI styles such as primary control, pending cue, confirmation, warning, and low-emphasis context
- migrate the current hierarchy helpers to that layer without changing visible text or adding a theme system
- keep style-focused tests asserting semantic roles through the helper outputs or stable role expectations

Follow-up: `RIOTBOX-258`

## Boundary Review

- Architecture and boundaries: the reviewed code still follows the app-level projection path and does not introduce a second action, queue, Scene, or persistence model.
- Design pattern consistency: the new lines preserve the existing `Line` / `Span` rendering pattern, but the raw style literals should be centralized before more hierarchy slices land.
- Technical debt and maintainability: the main debt is ordering and style-role drift, not a correctness break.
- Cross-module coupling: the TUI stays read-only over app/runtime view state; no new core coupling was found in this seam.
- Security and performance: no security-sensitive behavior was added; the extra span construction is small UI work and does not touch the realtime audio path.

## Review Notes

- No blocking architecture regression was found in the Jam hierarchy seam.
- Queue / commit semantics remain visible and contract-aligned.
- The next useful work should stay small: first correct the primary `Next` ordering, then centralize semantic style helpers before more color hierarchy is added.
