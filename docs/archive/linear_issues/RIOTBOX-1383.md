# `RIOTBOX-1383` P023: Render release-demo review worklist

- Ticket: `RIOTBOX-1383`
- Title: `P023: Render release-demo review worklist`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1383/p023-render-release-demo-review-worklist`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-05`
- Started: `2026-07-05`
- Finished: `2026-07-05`
- Branch: `feature/riotbox-1383-p023-render-release-demo-review-worklist`
- Linear branch: `feature/riotbox-1383-p023-render-release-demo-review-worklist`
- Assignee: `Markus`
- Labels: None
- PR: `#1347 (https://github.com/marang/riotbox/pull/1347)`
- Merge commit: `5751a3e003e6887cc623cd2f05c3a5cf28c01ede`
- Deleted from Linear: `2026-07-05`
- Verification: `just sound-quality-readiness-report-smoke via scripts/run_compact.sh; python3 -m py_compile scripts/generate_sound_quality_readiness_report.py; git diff --check; GitHub rust-ci passed on PR #1347`
- Docs touched: `Justfile; scripts/generate_sound_quality_readiness_report.py`
- Follow-ups: `Use the rendered review worklist to drive structured human verdict imports for pad_noise, bad_timing, weak_source, and sparse_drums coverage.`

## Why This Ticket Existed

P023 readiness had concrete review candidates but the actionable listening work was still too buried in JSON for a musician or reviewer.

## What Shipped

- Rendered a priority-ordered Release-Demo Review Worklist in the sound-quality readiness Markdown report, including verdict target, candidate reasons, artifact refs, and listening questions.

## Notes

- No rendered audio changed; candidates remain unverified and non-quality-proof until structured listening records verdicts.
