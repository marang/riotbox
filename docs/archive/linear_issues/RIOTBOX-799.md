# `RIOTBOX-799` Turn representative source showcase from technical proof into musically convincing demo

- Ticket: `RIOTBOX-799`
- Title: `Turn representative source showcase from technical proof into musically convincing demo`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-799/turn-representative-source-showcase-from-technical-proof-into`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-799-musical-showcase-gate`
- Linear branch: `feature/riotbox-799-turn-representative-source-showcase-from-technical-proof`
- Assignee: `Markus`
- Labels: `Improvement`, `benchmark`
- PR: `#795 (https://github.com/marang/riotbox/pull/795)`
- Merge commit: `562b9d98e5342353942e80970a2973a51812127c`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-799-showcase-after.log scripts/generate_representative_source_showcase.sh /tmp/riotbox-799-showcase-after local-riotbox-799-after 4.0 4`; `scripts/run_compact.sh /tmp/riotbox-799-just-representative-showcase.log just representative-source-showcase /tmp/riotbox-799-just-showcase local-riotbox-799-just 4.0 4`; `scripts/run_compact.sh /tmp/riotbox-799-musical-quality-target-rerun.log just representative-source-showcase-musical-quality /tmp/riotbox-799-showcase-after`; `scripts/run_compact.sh /tmp/riotbox-799-feral-grid-tests.log cargo test -p riotbox-audio --bin feral_grid_pack`; `scripts/run_compact.sh /tmp/riotbox-799-syncopated-smoke.log just syncopated-source-showcase-smoke`; `scripts/run_compact.sh /tmp/riotbox-799-audio-qa-ci.log just audio-qa-ci`; `scripts/run_compact.sh /tmp/riotbox-799-clippy-audio.log cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings`; `scripts/run_compact.sh /tmp/riotbox-799-fmt.log cargo fmt --check`; `scripts/run_compact.sh /tmp/riotbox-799-pycompile-final.log python3 -m py_compile scripts/validate_representative_showcase_musical_quality.py`; `git diff --check`; `GitHub Actions Rust CI run 1924 passed on 2d0e916514644037af12b178f22204d69c312a95`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`; `docs/benchmarks/representative_source_showcase_2026-05-07.md`; `docs/reviews/representative_showcase_musical_quality_2026-05-14.md`; `docs/README.md`
- Follow-ups: `RIOTBOX-801`

## Why This Ticket Existed

The representative source showcase could prove source response while still sounding like a QA artifact instead of a musician-facing Riotbox example.

## What Shipped

- Added a representative-showcase musical-quality gate, wired it into the local showcase generator, documented the boundary, recorded a review note, and raised generated-support TR-909 presence while keeping source-first masking controlled.

## Notes

- Selected local candidate after the change: tonal_hook_chop/late with support/source ratio 0.272528, source-first ratio 0.063978, and bar similarity 0.962824. The gate is a review aid, not automatic taste scoring.
