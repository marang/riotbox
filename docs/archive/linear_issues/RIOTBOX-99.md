# `RIOTBOX-99` Curate Jam action language into clearer primary and secondary gesture vocabularies

- Ticket: `RIOTBOX-99`
- Title: `Curate Jam action language into clearer primary and secondary gesture vocabularies`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-99/curate-jam-action-language-into-clearer-primary-and-secondary-gesture`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-99-gesture-language`
- Linear branch: `feature/riotbox-99-curate-jam-action-language-into-clearer-primary-and`
- Assignee: `Markus`
- Labels: `None`
- PR: `#91`
- Merge commit: `260ec7d45dc2d88a7ff266a967df70adc07cd07c`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#259`
- Docs touched: `README.md`, `docs/jam_recipes.md`, `docs/research_decision_log.md`, `docs/screenshots/jam_gesture_language_baseline.txt`, `docs/README.md`
- Follow-ups: `RIOTBOX-97`

## Why This Ticket Existed

The perform-first Jam surface was already structurally reduced, but it was still speaking too often in engine-model terms. Riotbox needed the main play surface to read more like an instrument without changing the underlying action lexicon or opening a second inspect architecture too early.

## What Shipped

- rewrote the outward-facing Jam footer, help overlay, status messages, and pending/landed summaries toward clearer gesture language like `voice`, `scene jump`, `follow`, `hit`, and `push`
- updated the MC-202 Jam card to use the same perform-facing vocabulary while keeping deep `Log` diagnostics technical
- added a normalized Jam shell baseline for the new gesture-language surface
- followed up with a dedicated `docs/jam_recipes.md` guide and linked it from the root README so the first-run path no longer had to teach the entire shell inline

## Notes

- this slice deliberately kept the internal action model and command ids unchanged; the change was presentation-only on the perform-first surface
- the recipe-guide follow-up was added on the same branch after live user feedback that the minimal `Space -> f -> c -> 2` path was too narrow for actual learning
- the next honest UX question is now `RIOTBOX-97`: whether a bounded Jam inspect mode can increase confidence without re-bloating the default performance surface
