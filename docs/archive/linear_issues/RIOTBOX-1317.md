# `RIOTBOX-1317` P023: Strengthen W-30 hook/chop policy for routed weak outputs

- Ticket: `RIOTBOX-1317`
- Title: `P023: Strengthen W-30 hook/chop policy for routed weak outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1317/p023-strengthen-w-30-hookchop-policy-for-routed-weak-outputs`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1317-p023-strengthen-w-30-hookchop-policy-for-routed-weak-outputs`
- Linear branch: `feature/riotbox-1317-p023-strengthen-w-30-hookchop-policy-for-routed-weak-outputs`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1291 (https://github.com/marang/riotbox/pull/1291)`
- Merge commit: `5812ad665bafd79074dee66d373d6d4bcc11749a`
- Deleted from Linear: `2026-06-29`
- Verification: `GitHub rust-ci passed`; `python3 -m py_compile scripts/generate_dense_break_performance_pack.py`; `git diff --check`; `just dense-break-performance-pack-smoke artifacts/audio_qa/local-riotbox-1317-dense-smoke-final`; `just professional-source-wav-pack-smoke artifacts/audio_qa/local-riotbox-1317-pro-source-smoke-2`; `just pro-pressure-source-matrix-smoke artifacts/audio_qa/local-riotbox-1317-pro-pressure-matrix`; `just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1317-weak-routing`; `just sound-quality-readiness-report-smoke artifacts/audio_qa/local-riotbox-1317-readiness`; `just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1317-prof-suite`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The P023 readiness report routed the top weak-output production fix to W-30 hook/chop policy: dense and tonal outputs could still read as generic support or hookless chop.

## What Shipped

- Stricter dense/tonal W-30 hook/chop contract with at least four source offsets, seven riff hits, two reverse gestures, expanded source-derived riff starts, tertiary response hits, and roadmap/audio-QA documentation.

## Notes

- None
