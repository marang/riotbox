# `RIOTBOX-1023` Tighten representative showcase musical-depth gates

- Ticket: `RIOTBOX-1023`
- Title: `Tighten representative showcase musical-depth gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1023/tighten-representative-showcase-musical-depth-gates`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-29`
- Started: `2026-05-29`
- Finished: `2026-05-29`
- Branch: `feature/riotbox-1023-mc202-representative-quality-gate`
- Linear branch: `feature/riotbox-1023-tighten-representative-showcase-musical-depth-gates`
- Assignee: `Markus`
- Labels: None
- PR: `#1010 (https://github.com/marang/riotbox/pull/1010)`
- Merge commit: `fc1353759d45b6bae909159310917bcce155263c`
- Deleted from Linear: `2026-05-29`
- Verification: `python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py scripts/validate_source_showcase_diversity.py; just representative-source-showcase-musical-quality-fixtures; just source-showcase-diversity-validator-fixtures; just source-showcase-diversity-report-fixtures; cargo test -p riotbox-audio --bin feral_grid_pack; scripts/generate_representative_source_showcase.sh /tmp/riotbox-1023-showcase-fixed3 local-riotbox-1023-fixed3 8.0 4; just audio-qa-ci; just ci; GitHub Actions Rust CI run 2550`
- Docs touched: `docs/reviews/p013_mc202_representative_quality_gate_review_2026-05-29.md; docs/specs/audio_qa_workflow_spec.md; docs/README.md`
- Follow-ups: `Continue P013 with the next all-lane depth slice after confirming no open P013 Linear issues remain.`

## Why This Ticket Existed

P013 needed representative showcase gates to prove MC-202 musical depth and avoid false positives in source-diversity and observer/audio correlation.

## What Shipped

- Enforced MC-202 bass-pressure, phrase-variation, and source-grid quality gates; raised bounded offline MC-202 bass-pressure output; fixed low-RMS/high-spectral source-diversity handling; selected Source Timing-compatible observer correlation for representative showcase generation.

## Notes

- None
