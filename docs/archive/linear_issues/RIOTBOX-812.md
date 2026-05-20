# `RIOTBOX-812` Extract feral grid pack manifest ownership into a real module

- Ticket: `RIOTBOX-812`
- Title: `Extract feral grid pack manifest ownership into a real module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-812/extract-feral-grid-pack-manifest-ownership-into-a-real-module`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-812-feral-grid-manifest-module`
- Linear branch: `feature/riotbox-812-extract-feral-grid-pack-manifest-ownership-into-a-real`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#807 (https://github.com/marang/riotbox/pull/807)`
- Merge commit: `723cecdbd8b789c62050e24d6e8b58cf453d812d`
- Verification: `cargo test -p riotbox-audio --bin feral_grid_pack; just syncopated-source-showcase-smoke; scripts/generate_representative_source_showcase.sh /tmp/riotbox-812-showcase local-riotbox-812 4.0 4; cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings; just audio-qa-ci; just ci; GitHub Actions Rust CI #1960 passed`
- Docs touched: `None`
- Follow-ups: `None for this slice.`

## Why This Ticket Existed

Reduce P013 feral_grid_pack review cost by converting the next touched manifest hotspot from textual include ownership into a real semantic module boundary.

## What Shipped

- Extracted listening-pack manifest structs and JSON writing helpers into crates/riotbox-audio/src/bin/feral_grid_pack/manifest.rs while preserving manifest JSON shape and audio/showcase output behavior.

## Notes

- Linear deletion not performed because LINEAR_API_TOKEN is not available in this session.
