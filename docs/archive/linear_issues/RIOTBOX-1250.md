# `RIOTBOX-1250` P023: Group weak-output routes into production-fix candidates

- Ticket: `RIOTBOX-1250`
- Title: `P023: Group weak-output routes into production-fix candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1250/p023-group-weak-output-routes-into-production-fix-candidates`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-13`
- Started: `2026-06-13`
- Finished: `2026-06-13`
- Branch: `feature/riotbox-1250-p023-weak-output-fix-candidates`
- Linear branch: `feature/riotbox-1250-p023-group-weak-output-routes-into-production-fix-candidates`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1225 (https://github.com/marang/riotbox/pull/1225)`
- Merge commit: `00ff7e95`
- Deleted from Linear: `2026-06-13`
- Verification: `python3 -m py_compile scripts/route_weak_output_fixes.py scripts/generate_sound_quality_readiness_report.py`; `just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1250-weak-routing`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1250-readiness`; `just professional-output-suite-smoke`; `just audio-qa-ci`; `just ci`; `GitHub rust-ci pass on PR #1225`
- Docs touched: `docs/benchmarks/weak_output_fix_routing_v1_2026-06-05.md`, `docs/benchmarks/sound_quality_readiness_report_v1_2026-06-12.md`
- Follow-ups: `None`

## Why This Ticket Existed

P023 needed weak-output diagnostics to become reliable production planning inputs. Routed weak/fail reports already identified categories, but stale grouped candidate data could still mislead the next sound-improvement slice.

## What Shipped

- Weak-output routing now emits production_fix_summary with grouped categories, recurring fix categories, case-reference counts, primary-case counts, and top candidate category.
- Added --validate-report to reject stale candidate counts, stale grouped summaries, unknown case ids, duplicate candidate categories, stale scores, stale artifact/source-family refs, and accidental quality-proof claims.
- P023 sound-quality readiness now surfaces and validates the weak-output production_fix_summary.
- Justfile fixtures now cover unknown routes, stale candidate counts, unknown candidate cases, stale summaries, and duplicate candidate categories.

## Notes

- This remains diagnostic actionability only: human_verdict stays unverified and quality_proof stays false.
