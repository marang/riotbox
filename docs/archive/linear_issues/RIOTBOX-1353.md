# `RIOTBOX-1353` P023: Strengthen destructive gesture from weak-output routing

- Ticket: `RIOTBOX-1353`
- Title: `P023: Strengthen destructive gesture from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1353/p023-strengthen-destructive-gesture-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1353-p023-strengthen-destructive-gesture-from-weak-output-routing`
- Linear branch: `feature/riotbox-1353-p023-strengthen-destructive-gesture-from-weak-output-routing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1317 (https://github.com/marang/riotbox/pull/1317)`
- Merge commit: `7795b3baa6e599109598d825da7c9f47b48dda16`
- Deleted from Linear: `2026-06-30`
- Verification: `python -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_destructive_variation_professional.py`; `cargo fmt --check`; `cargo test -p riotbox-audio --bin feral_grid_pack`; `cargo test -p riotbox-core --bin source_timing_fixture_report`; `just destructive-variation-professional-smoke`; `just pro-pressure-source-matrix-smoke`; `just professional-source-wav-pack-smoke`; `just professional-output-suite-smoke`; `just sound-quality-readiness-report-smoke`; `just audio-qa-ci`; `just ci`; `GitHub Actions rust-ci passed on PR #1317`
- Docs touched: `None`
- Follow-ups: `Large generator and included Rust shard review hotspots remain candidates for semantic module splits when a real ownership boundary emerges.`

## Why This Ticket Existed

Weak-output routing still flagged destructive gesture output as too weak after nearby W-30 and MC-202 proof hardening. This ticket existed to make dropout/stutter/restore impact harder to pass accidentally while preserving honest diagnostic labeling: scripted source-derived evidence remains useful for regression, but not a release-grade musical proof.

## What Shipped

- Raised destructive-variation professional thresholds for stutter impact, restore impact, restore-vs-pressure size, dropout-stutter RMS, and restore RMS.
- Strengthened dense-break tail shaping so the dropout/stutter/restore render produces a more stage-meaningful cut, bite, and restore hit for musician review.
- Replaced source-family magic strings in the dense-break generator with named constants and centralized Rust manifest/QA contract strings for source-derived and primitive origins plus MC-202/TR-909/W-30 proof roles.

## Notes

- Human verdict remains unverified; shipped evidence is stronger diagnostic proof, not a release-grade musical pass claim.
