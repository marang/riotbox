# `RIOTBOX-1014` P012: Record Beat20 downbeat feasibility guardrail

- Ticket: `RIOTBOX-1014`
- Title: `P012: Record Beat20 downbeat feasibility guardrail`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1014/p012-record-beat20-downbeat-feasibility-guardrail`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1014-p012-record-beat20-downbeat-feasibility-guardrail`
- Linear branch: `feature/riotbox-1014-p012-record-beat20-downbeat-feasibility-guardrail`
- Assignee: `Markus`
- Labels: `Docs`, `benchmark`, `timing`
- PR: `#997 (https://github.com/marang/riotbox/pull/997)`
- Merge commit: `0ebb41675e98ff9cd79d5a37924b08db6e89ebe4`
- Deleted from Linear: `2026-05-27`
- Verification: `just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md (passed 2026-05-27)`; `git diff --check (passed 2026-05-27)`
- Docs touched: `docs/reviews/p012_beat20_downbeat_feasibility_2026-05-27.md`, `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Beat20 is the current useful-but-ambiguous real-source P012 timing row. The local report showed stable BPM/beat evidence, but downbeat margin 0.005, three alternate phases, and transient-only anchors. A bounded low-band check did not create enough phase evidence to justify changing analyzer policy, so the repo needed a durable guardrail against premature promotion.

## What Shipped

- Added a focused Beat20 downbeat feasibility review note with current report values and low-band phase-margin results.
- Recorded decision RBX-058: keep Beat20-like rows manual-confirm-only / ambiguous-downbeat until stronger musical anchor or phase evidence exists.
- Left analyzer, Session, UI, realtime, and audio-render behavior unchanged; no audible behavior change was claimed.

## Notes

- PR #997 merged as 0ebb41675e98ff9cd79d5a37924b08db6e89ebe4; PR metadata was confirmed through the GitHub connector because unauthenticated REST was rate-limited during archive generation.
