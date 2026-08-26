# `RIOTBOX-1477` Publish source-matched reconstructable product stems

- Ticket: `RIOTBOX-1477`
- Title: `Publish source-matched reconstructable product stems`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1477/publish-source-matched-reconstructable-product-stems`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-26`
- Started: `2026-08-26`
- Finished: `2026-08-26`
- Branch: `feature/riotbox-1477-publish-source-matched-reconstructable-product-stems`
- Linear branch: `feature/riotbox-1477-publish-source-matched-reconstructable-product-stems`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1488 (https://github.com/marang/riotbox/pull/1488)`
- Merge commit: `5c5caa1aa0fc465d2700457fc4f9256bbf2dffb5`
- Deleted from Linear: `2026-08-26`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack (44/44)`; `python3 -m unittest scripts.test_validate_product_stem_handoff (8/8)`; `synthetic product-stem double-render handoff and existing product-mix reproducibility smoke`; `cargo clippy --all-targets --all-features -- -D warnings`; `just ci and GitHub rust-ci`
- Docs touched: `README.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/session_file_spec.md`, `docs/reviews/riotbox_1477_product_stem_handoff_2026-08-26.md`
- Follow-ups: `RIOTBOX-1036 retains the wider musician stem-package, live-recording, and DAW workflow; the next bounded slice should consume canonical Core/Session lineage rather than treating this developer proof as committed state.`

## Why This Ticket Existed

The existing raw Feral-grid lanes entered a shared nonlinear product bus and could not honestly reconstruct the approved full mix, while the hardcoded local-CI stem fixture was never valid musician material.

## What Shipped

- Preserved the approved full_grid_mix byte-for-byte and attributed its post-bus output across typed TR-909 drums, W-30 music, and MC-202 bass contribution stems.
- Added a versioned development-only double-render handoff with frozen PCM-sum reconstruction, source/manifest/grid/artifact hashes, contained atomic publication, and no-overwrite behavior.
- Kept the MC-202 primitive-renderer boundary explicit and left export.stem_package, Session receipts, TUI/Ghost, DAW, live recording, and release readiness unclaimed.

## Notes

- No registered Development source, Holdout, commercial reference, or human playback was used; the proof was source-free and synthetic.
